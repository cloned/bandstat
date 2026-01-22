//! Frequency band definitions

/// Frequency band with label and range
pub(crate) struct Band {
    pub(crate) label: &'static str,
    pub(crate) low_hz: f32,
    pub(crate) high_hz: f32,
}

/// Get the 14 standard frequency bands from DC to AIR
pub(crate) fn get_bands() -> Vec<Band> {
    vec![
        Band {
            label: "DC",
            low_hz: 0.0,
            high_hz: 20.0,
        },
        Band {
            label: "SUB1",
            low_hz: 20.0,
            high_hz: 40.0,
        },
        Band {
            label: "SUB2",
            low_hz: 40.0,
            high_hz: 60.0,
        },
        Band {
            label: "BASS",
            low_hz: 60.0,
            high_hz: 120.0,
        },
        Band {
            label: "UBAS",
            low_hz: 120.0,
            high_hz: 250.0,
        },
        Band {
            label: "LMID",
            low_hz: 250.0,
            high_hz: 500.0,
        },
        Band {
            label: "MID",
            low_hz: 500.0,
            high_hz: 1000.0,
        },
        Band {
            label: "UMID",
            low_hz: 1000.0,
            high_hz: 2000.0,
        },
        Band {
            label: "HMID",
            low_hz: 2000.0,
            high_hz: 4000.0,
        },
        Band {
            label: "PRES",
            low_hz: 4000.0,
            high_hz: 6000.0,
        },
        Band {
            label: "BRIL",
            low_hz: 6000.0,
            high_hz: 10000.0,
        },
        Band {
            label: "HIGH",
            low_hz: 10000.0,
            high_hz: 14000.0,
        },
        Band {
            label: "UHIG",
            low_hz: 14000.0,
            high_hz: 18000.0,
        },
        Band {
            label: "AIR",
            low_hz: 18000.0,
            high_hz: f32::MAX,
        },
    ]
}
