use ash::{extensions::khr, vk};

pub struct SurfaceWrapper {
    pub surface: vk::SurfaceKHR,
    surface_loader: khr::Surface,
}

impl SurfaceWrapper {
    pub fn init(
        window: &winit::window::Window,
        entry: &ash::Entry,
        instance: &ash::Instance,
    ) -> SurfaceWrapper {
        // load the surface
        // handles x11 or whatever OS specific drivers
        // this shit is terrible and nobody wants to do it, so lets use ash-window
        let surface = unsafe { ash_window::create_surface(entry, instance, window, None).unwrap() };
        let surface_loader = khr::Surface::new(entry, instance);

        SurfaceWrapper {
            surface,
            surface_loader,
        }
    }

    pub fn get_capabilities(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> Result<vk::SurfaceCapabilitiesKHR, vk::Result> {
        unsafe {
            self.surface_loader
                .get_physical_device_surface_capabilities(physical_device, self.surface)
        }
    }

    pub fn get_present_modes(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> Result<Vec<vk::PresentModeKHR>, vk::Result> {
        unsafe {
            self.surface_loader
                .get_physical_device_surface_present_modes(physical_device, self.surface)
        }
    }

    pub fn get_formats(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> Result<Vec<vk::SurfaceFormatKHR>, vk::Result> {
        unsafe {
            self.surface_loader
                .get_physical_device_surface_formats(physical_device, self.surface)
        }
    }

    pub fn get_physical_device_surface_support(
        &self,
        physical_device: vk::PhysicalDevice,
        queuefamilyindex: usize,
    ) -> Result<bool, vk::Result> {
        unsafe {
            self.surface_loader.get_physical_device_surface_support(
                physical_device,
                queuefamilyindex as u32,
                self.surface,
            )
        }
    }

    pub fn choose_present_mode(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> Result<vk::PresentModeKHR, vk::Result> {
        let present_modes = self.get_present_modes(physical_device)?;
        // todo: prefere immediate over fifo
        Ok(if present_modes.contains(&vk::PresentModeKHR::MAILBOX) {
            vk::PresentModeKHR::MAILBOX
        } else if present_modes.contains(&vk::PresentModeKHR::IMMEDIATE) {
            vk::PresentModeKHR::IMMEDIATE
        } else {
            vk::PresentModeKHR::FIFO
        })
    }

    pub fn choose_format(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> Result<vk::SurfaceFormatKHR, vk::Result> {
        let formats = self.get_formats(physical_device)?;
        let optimal = formats.iter().find(|x| {
            (x.format == vk::Format::B8G8R8A8_SRGB || x.format == vk::Format::R8G8B8A8_SRGB)
                && x.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        });
        Ok(if let Some(optimal) = optimal {
            *optimal
        } else {
            formats[0]
        })
    }
}

impl Drop for SurfaceWrapper {
    fn drop(&mut self) {
        unsafe {
            self.surface_loader.destroy_surface(self.surface, None);
        }
    }
}
