use ash::vk;

fn main() {
    // Initialize Vulkan instance
    let entry = ash::Entry::new().unwrap();
    let app_name = std::ffi::CString::new("My Vulkan App").unwrap();
    let engine_name = std::ffi::CString::new("My Vulkan Engine").unwrap();
    let app_info = vk::ApplicationInfo::builder()
        .application_name(&app_name)
        .application_version(1)
        .engine_name(&engine_name)
        .engine_version(1)
        .api_version(vk::make_version(1, 2, 0));
    let instance_extensions = [ash::extensions::ext::DebugUtils::name().as_ptr()];
    let instance_create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_extension_names(&instance_extensions);
    let instance = unsafe { entry.create_instance(&instance_create_info, None).unwrap() };

    // Select a physical device
    let physical_devices = unsafe { instance.enumerate_physical_devices().unwrap() };
    let physical_device = physical_devices[0];

    // Create a logical device and graphics queue
    let queue_family_index = 0;
    let queue_priorities = [1.0];
    let queue_create_info = vk::DeviceQueueCreateInfo::builder()
        .queue_family_index(queue_family_index)
        .queue_priorities(&queue_priorities);
    let device_extensions = [ash::extensions::khr::Swapchain::name().as_ptr()];
    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&[queue_create_info.build()])
        .enabled_extension_names(&device_extensions);
    let device = unsafe { instance.create_device(physical_device, &device_create_info, None).unwrap() };
    let graphics_queue = unsafe { device.get_device_queue(queue_family_index as u32, 0) };

    // Create a command pool and command buffer
    let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
        .queue_family_index(queue_family_index);
    let command_pool = unsafe { device.create_command_pool(&command_pool_create_info, None).unwrap() };
    let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(command_pool)
        .command_buffer_count(1);
    let command_buffers = unsafe { device.allocate_command_buffers(&command_buffer_allocate_info).unwrap() };
    let command_buffer = command_buffers[0];

    // Begin command buffer recording
    let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder();
    unsafe { device.begin_command_buffer(command_buffer, &command_buffer_begin_info).unwrap() };

    // Create vertex and fragment shaders
    let vertex_shader_code = include_bytes!("triangle.vert.spv");
    let vertex_shader_module_create_info = vk::ShaderModuleCreateInfo::builder()
        .code(&vertex_shader_code[..]);
    let vertex_shader_module = unsafe { device.create_shader_module(&vertex_shader_module_create_info, None).unwrap() };
    let fragment_shader_code = include_bytes!("triangle.frag.spv");
    let fragment_shader_module_create_info = vk::ShaderModuleCreateInfo::builder()
        .code(&fragment_shader_code[..]);
    let fragment_shader_module = unsafe { device.create_shader_module(&fragment_shader_module_create_info, None).unwrap() };

    // Create vertex input binding and attribute descriptions
    let binding_description = vk::VertexInputBindingDescription::builder()
        .binding(0)
        .input_rate(vk::VertexInputRate::VERTEX)
        .stride(3 * std::mem::size_of::<f32>() as u32);
    let attribute_descriptions = [
        vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(0)
            .build(),
    ];

    // Create graphics pipeline layout
    let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder();
    let pipeline_layout = unsafe { device.create_pipeline_layout(&pipeline_layout_create_info, None).unwrap() };

    // Create graphics pipeline
    let pipeline_shader_stages = [
        vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vertex_shader_module)
            .name(std::ffi::CString::new("main").unwrap().as_ptr())
            .build(),
        vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(fragment_shader_module)
            .name(std::ffi::CString::new("main").unwrap().as_ptr())
            .build(),
    ];
    let pipeline_vertex_input_state_create_info = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(&[binding_description.build()])
        .vertex_attribute_descriptions(&attribute_descriptions);
    let pipeline_input_assembly_state_create_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
    let pipeline_viewport_state_create_info = vk::PipelineViewportStateCreateInfo::builder()
        .viewport_count(1)
        .scissor_count(1);
    let pipeline_rasterization_state_create_info = vk::PipelineRasterizationStateCreateInfo::builder()
        .polygon_mode(vk::PolygonMode::FILL)
        .cull_mode(vk::CullModeFlags::BACK)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .line_width(1.0);
    let pipeline_multisample_state_create_info = vk::PipelineMultisampleStateCreateInfo::builder()
        .rasterization_samples(vk::SampleCountFlags::TYPE_1);
    let pipeline_color_blend_attachment_state = vk::PipelineColorBlendAttachmentState::builder()
        .color_write_mask(vk::ColorComponentFlags::all())
        .build();
    let pipeline_color_blend_state_create_info = vk::PipelineColorBlendStateCreateInfo::builder()
        .attachments(&[pipeline_color_blend_attachment_state]);
    let pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(&pipeline_shader_stages)
        .vertex_input_state(&pipeline_vertex_input_state_create_info)
        .input_assembly_state(&pipeline_input_assembly_state_create_info)
        .viewport_state(&pipeline_viewport_state_create_info)
        .rasterization_state(&pipeline_rasterization_state_create_info)
        .multisample_state(&pipeline_multisample_state_create_info)
        .color_blend_state(&pipeline_color_blend_state_create_info)
        .layout(pipeline_layout)
        .render_pass(render_pass)
        .subpass(0);
    let pipeline = unsafe { device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_create_info.build()], None).unwrap()[0] };

    // Create framebuffers
    let framebuffer_create_info = vk::FramebufferCreateInfo::builder()
        .render_pass(render_pass)
        .attachments(&[image_views[image_index as usize]])
        .width(swapchain_extent.width)
        .height(swapchain_extent.height)
        .layers(1);
    let framebuffer = unsafe { device.create_framebuffer(&framebuffer_create_info, None).unwrap() };

    // Create command buffers
    let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(1);
    let command_buffers = unsafe { device.allocate_command_buffers(&command_buffer_allocate_info).unwrap() };
    let command_buffer = command_buffers[0];
    let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);
    unsafe { device.begin_command_buffer(command_buffer, &command_buffer_begin_info).unwrap() };

    // Begin render pass
    let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
        .render_pass(render_pass)
        .framebuffer(framebuffer)
        .render_area(vk::Rect2D::builder().offset(vk::Offset2D::default()).extent(swapchain_extent).build())
        .clear_values(&[vk::ClearValue { color: vk::ClearColorValue { float32: [0.0, 0.0, 0.0, 1.0] } }]);
    unsafe {
        device.cmd_begin_render_pass(command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);
        device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline);
        device.cmd_bind_vertex_buffers(command_buffer, 0, &[vertex_buffer], &[0]);
        device.cmd_draw(command_buffer, 3, 1, 0, 0);
        device.cmd_end_render_pass(command_buffer);
        device.end_command_buffer(command_buffer).unwrap();
    }

    // Submit command buffer to graphics queue
    let submit_info = vk::SubmitInfo::builder()
        .command_buffers(&[command_buffer]);
    let fence_create_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
    let fence = unsafe { device.create_fence(&fence_create_info, None).unwrap() };
    unsafe {
        device.queue_submit(graphics_queue, &[submit_info.build()], fence);
    }

    // Wait for graphics queue to finish rendering
    unsafe {
        device.wait_for_fences(&[fence], true, std::u64::MAX).unwrap();
        device.destroy_fence(fence, None);
        device.free_command_buffers(command_pool, &[command_buffer]);
        device.destroy_framebuffer(framebuffer, None);
        device.destroy_pipeline(pipeline, None);
        device.destroy_pipeline_layout(pipeline_layout, None);
        device.destroy_shader_module(vertex_shader_module, None);
        device.destroy_shader_module(fragment_shader_module, None);
        device.destroy_buffer(vertex_buffer, None);
        device.free_memory(vertex_buffer_memory, None);
        device.destroy_semaphore(render_finished_semaphore, None);
        device.destroy_semaphore(image_available_semaphore, None);
    }
}