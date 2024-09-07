use fraction::Fraction;
use log::info;
#[cfg(windows)]
use windows::core::PCWSTR;
#[cfg(windows)]
use windows::Win32::Graphics::Gdi::{
    EnumDisplayDevicesW, EnumDisplaySettingsW, DEVMODEW, DISPLAY_DEVICEW, DISPLAY_DEVICE_ACTIVE,
    DISPLAY_DEVICE_PRIMARY_DEVICE, ENUM_CURRENT_SETTINGS, ENUM_DISPLAY_SETTINGS_MODE,
};

#[derive(Eq, PartialEq, Debug, PartialOrd, Ord)]
pub struct Resolution {
    pub w: u32,
    pub h: u32,
    pub freqs: Vec<u32>,
}

pub struct Screen {
    pub resolutions: Vec<Resolution>,
    pub current_resolution: Option<Resolution>,
}

impl Screen {
    pub fn position(&self, w: u32, h: u32) -> Option<usize> {
        let (w, h) = if w == 0 || h == 0 {
            match &self.current_resolution {
                Some(r) => (r.w, r.h),
                None => (0, 0),
            }
        } else {
            (w, h)
        };

        self.resolutions.iter().position(|s| s.w == w && s.h == h)
    }

    pub fn refresh_rates_len(&self, resolution_position: usize) -> Option<usize> {
        self.resolutions
            .get(resolution_position)
            .map(|resolution| resolution.freqs.len())
    }

    pub fn refresh_rate_position(&self, resolution_position: usize, freq: u32) -> Option<usize> {
        let freq = if freq == 0 {
            match &self.current_resolution {
                Some(r) => r.freqs[0],
                None => 0,
            }
        } else {
            freq
        };
        self.resolutions
            .get(resolution_position)
            .and_then(|resolution| resolution.freqs.iter().position(|f| *f == freq))
    }

    #[cfg(windows)]
    pub fn list_screens_resolutions() -> Screen {
        let mut resolutions: Vec<Resolution> = vec![];
        let mut current_resolution = None;
        let mut dev_num = 0;
        loop {
            let mut display_device = DISPLAY_DEVICEW {
                cb: std::mem::size_of::<DISPLAY_DEVICEW>() as u32,
                ..DISPLAY_DEVICEW::default()
            };
            unsafe {
                if !EnumDisplayDevicesW(PCWSTR::null(), dev_num, &mut display_device, 0).as_bool() {
                    break;
                }
            }

            if (display_device.StateFlags & DISPLAY_DEVICE_ACTIVE) != 0
                && (display_device.StateFlags & DISPLAY_DEVICE_PRIMARY_DEVICE) != 0
            {
                let mut dev_mode = DEVMODEW {
                    dmSize: std::mem::size_of::<DEVMODEW>() as u16,
                    ..DEVMODEW::default()
                };
                unsafe {
                    if !EnumDisplaySettingsW(
                        PCWSTR::from_raw(display_device.DeviceName.as_ptr()),
                        ENUM_CURRENT_SETTINGS,
                        &mut dev_mode,
                    )
                    .as_bool()
                    {
                        break;
                    }
                }
                let current_ratio = Fraction::new(dev_mode.dmPelsWidth, dev_mode.dmPelsHeight);
                info!("Current screen ratio: {}", current_ratio);
                current_resolution = Some(Resolution {
                    w: dev_mode.dmPelsWidth,
                    h: dev_mode.dmPelsHeight,
                    freqs: vec![dev_mode.dmDisplayFrequency],
                });
                let mut imode_num = 0;

                loop {
                    let mut dev_mode = DEVMODEW {
                        dmSize: std::mem::size_of::<DEVMODEW>() as u16,
                        ..DEVMODEW::default()
                    };
                    unsafe {
                        if !EnumDisplaySettingsW(
                            PCWSTR::from_raw(display_device.DeviceName.as_ptr()),
                            ENUM_DISPLAY_SETTINGS_MODE(imode_num),
                            &mut dev_mode,
                        )
                        .as_bool()
                        {
                            break;
                        }
                    }
                    let ratio = Fraction::new(dev_mode.dmPelsWidth, dev_mode.dmPelsHeight);

                    if dev_mode.dmBitsPerPel >= 32 && current_ratio == ratio {
                        if let Some(position) = resolutions.iter().position(|s| {
                            s.w == dev_mode.dmPelsWidth && s.h == dev_mode.dmPelsHeight
                        }) {
                            if !resolutions[position]
                                .freqs
                                .contains(&dev_mode.dmDisplayFrequency)
                            {
                                resolutions[position]
                                    .freqs
                                    .push(dev_mode.dmDisplayFrequency);
                                resolutions[position].freqs.sort()
                            }
                        } else {
                            resolutions.push(Resolution {
                                w: dev_mode.dmPelsWidth,
                                h: dev_mode.dmPelsHeight,
                                freqs: vec![dev_mode.dmDisplayFrequency],
                            });
                        }
                    }

                    imode_num += 1
                }
            }

            dev_num += 1;
        }

        resolutions.sort();

        Screen {
            resolutions,
            current_resolution,
        }
    }

    #[cfg(unix)]
    pub fn list_screens_resolutions() -> Screen {
        Screen {
            resolutions: vec![],
            current_resolution: None,
        }
    }
}
