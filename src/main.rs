use std::io::{Error, ErrorKind, Result};
use std::sync::Arc;

use vulkano::device::{Device, DeviceExtensions};
use vulkano::format::Format;
use vulkano::image::attachment::AttachmentImage;
use vulkano::instance::{layers_list, Instance, PhysicalDevice, PhysicalDevicesIter};
use vulkano::swapchain::PresentMode;
use vulkano::swapchain::Surface;
use winit::window::Window;
use winit::window::WindowBuilder;

type WindowSurface = Surface<Window>;

fn normalize_error<E: std::fmt::Display>(error: E) -> Error {
  Error::new(ErrorKind::Other, format!("{}", error))
}

fn main() -> Result<()> {
  env_logger::init();
  let extensions = vulkano_win::required_extensions();
  let layers = layers_list()
    .map_err(normalize_error)?
    .map(|l| String::from(l.name()))
    .collect::<Vec<String>>();

  let events = winit::event_loop::EventLoop::new();
  let inst =
    Instance::new(None, &extensions, layers.iter().map(|l| l.as_str())).map_err(normalize_error)?;
  let devs = PhysicalDevice::enumerate(&inst);
  let wind = WindowBuilder::new()
    .with_title("segfault-1554")
    .build(&events)
    .map_err(normalize_error)?;
  let surf = vulkano_win::create_vk_surface(wind, inst.clone()).map_err(normalize_error)?;
  let phys = find_physical_device(devs, &surf).ok_or(std::io::Error::from_raw_os_error(22))?;

  let quef = phys
    .queue_families()
    .find(|&fam| fam.supports_graphics() && surf.is_supported(fam).unwrap_or(false))
    .ok_or(std::io::Error::from_raw_os_error(22))?;

  let phsf = phys.supported_features();
  let dext = DeviceExtensions {
    khr_swapchain: true,
    ..DeviceExtensions::supported_by_device(phys)
  };
  let (devi, _) = Device::new(phys, phsf, &dext, vec![(quef, 0.5)]).map_err(normalize_error)?;
  let dimensions = surf.window().inner_size();

  log::debug!("creating attchment image");
  AttachmentImage::new(devi.clone(), dimensions.into(), Format::D16Unorm)
    .map_err(normalize_error)?;
  log::debug!("attchment image ready");

  Ok(())
}

fn find_physical_device<'a>(
  devices: PhysicalDevicesIter<'a>,
  surface: &Arc<WindowSurface>,
) -> Option<PhysicalDevice<'a>> {
  for dev in devices {
    let fams = dev.queue_families();
    let rend = fams
      .filter(|fam| {
        let sg = fam.supports_graphics();
        // let sc = fam.supports_compute();
        sg
      })
      .next();
    let caps = surface.capabilities(dev).ok()?;

    if caps.present_modes.supports(PresentMode::Fifo) == false {
      continue;
    }

    if caps.supported_formats.len() < 1 {
      continue;
    }

    if let Some(_q) = rend {
      return Some(dev);
    }
  }

  None
}
