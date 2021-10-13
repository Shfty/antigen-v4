use crate::{
    antigen_hid::{GenericDesktopUsage, ReportDescriptor, Usage, UsagePage},
    devices::{T16000M_REPORT, TFRP_REPORT, TWCS_REPORT},
    parse_input,
};

#[test]
fn test_t16000m_input() {
    const T16000M_INPUT: [u8; 9] = [0x00, 0x00, 0x3F, 0x00, 0x20, 0x00, 0x20, 0xB2, 0xDE];

    let report_desc = ReportDescriptor::new(T16000M_REPORT.iter().copied());
    let result = parse_input(&report_desc, T16000M_INPUT.iter().copied());

    let (_, x_value) = result
        .iter()
        .find(|(data, _)| {
            data.global_state.usage_page == Some(UsagePage::GenericDesktop)
                && data.local_state.usage == Some(Usage::GenericDesktop(GenericDesktopUsage::X))
        })
        .expect("No X axis");

    let (_, y_value) = result
        .iter()
        .find(|(data, _)| {
            data.global_state.usage_page == Some(UsagePage::GenericDesktop)
                && data.local_state.usage == Some(Usage::GenericDesktop(GenericDesktopUsage::Y))
        })
        .expect("No Y axis");

    let (_, rz_value) = result
        .iter()
        .find(|(data, _)| {
            data.global_state.usage_page == Some(UsagePage::GenericDesktop)
                && data.local_state.usage == Some(Usage::GenericDesktop(GenericDesktopUsage::Rz))
        })
        .expect("No Rz axis");

    let (_, slider_value) = result
        .iter()
        .find(|(data, _)| {
            data.global_state.usage_page == Some(UsagePage::GenericDesktop)
                && data.local_state.usage
                    == Some(Usage::GenericDesktop(GenericDesktopUsage::Slider))
        })
        .expect("No Slider axis");

    let (_, buttons_value) = result
        .iter()
        .find(|(data, _)| data.global_state.usage_page == Some(UsagePage::Button))
        .expect("No Buttons");

    let (_, pov_hat_value) = result
        .iter()
        .find(|(data, _)| {
            data.global_state.usage_page == Some(UsagePage::GenericDesktop)
                && data.local_state.usage
                    == Some(Usage::GenericDesktop(GenericDesktopUsage::HatSwitch))
        })
        .expect("No POV Hat");

    let target_buttons_value = 0;
    let target_pov_hat_value = 15;
    let target_x_value = 8192;
    let target_y_value = 8192;
    let target_rz_value = 178;
    let target_slider_value = 222;

    assert!(
        *buttons_value == 0,
        "Buttons Value: {}, Target: {}",
        buttons_value,
        target_buttons_value
    );
    assert!(
        *pov_hat_value == 15,
        "POV Hat Value: {}, Target: {}",
        pov_hat_value,
        target_pov_hat_value
    );
    assert!(
        *x_value == target_x_value,
        "X Value: {}, Target: {}",
        x_value,
        target_x_value
    );
    assert!(
        *y_value == target_y_value,
        "Y Value: {}, Target: {}",
        y_value,
        target_y_value
    );
    assert!(
        *rz_value == target_rz_value,
        "Z Value: {}, Target: {}",
        rz_value,
        target_rz_value
    );
    assert!(
        *slider_value == target_slider_value,
        "Slider Value: {}, Target: {}",
        slider_value,
        target_slider_value
    );
}

#[test]
fn test_twcs_input() {
    const TWCS_INPUT: [u8; 64] = [
        0x01, 0xFE, 0x01, 0x09, 0x02, 0x00, 0x02, 0xFF, 0x03, 0xFF, 0x03, 0x00, 0x02, 0x75, 0x02,
        0xFC, 0x89, 0x08, 0x00, 0x00, 0xBF, 0x3F, 0x40, 0x3E, 0x5F, 0x32, 0x21, 0x3F, 0x95, 0x2B,
        0x82, 0x28, 0x1D, 0x3B, 0x4D, 0x28, 0xCD, 0xBA, 0xE7, 0xB7, 0x28, 0x28, 0x80, 0x79, 0x20,
        0x20, 0xA0, 0xA0, 0x2B, 0x2B, 0xA0, 0xA0, 0xB7, 0x55, 0xA7, 0x87, 0x1A, 0x09, 0xF5, 0x44,
        0x00, 0x00, 0x80, 0x02,
    ];

    let report_desc = ReportDescriptor::new(TWCS_REPORT.iter().copied());
    let result = parse_input(&report_desc, TWCS_INPUT.iter().copied());

    let (_, buttons_value) = result
        .iter()
        .find(|(data, _)| data.global_state.usage_page == Some(UsagePage::Button))
        .expect("No Buttons");

    let (_, pov_hat_value) = result
        .iter()
        .find(|(data, _)| {
            data.global_state.usage_page == Some(UsagePage::GenericDesktop)
                && data.local_state.usage
                    == Some(Usage::GenericDesktop(GenericDesktopUsage::HatSwitch))
        })
        .expect("No POV Hat");

    let (_, x_value) = result
        .iter()
        .find(|(data, _)| {
            data.global_state.usage_page == Some(UsagePage::GenericDesktop)
                && data.local_state.usage == Some(Usage::GenericDesktop(GenericDesktopUsage::X))
        })
        .expect("No X axis");

    let (_, y_value) = result
        .iter()
        .find(|(data, _)| {
            data.global_state.usage_page == Some(UsagePage::GenericDesktop)
                && data.local_state.usage == Some(Usage::GenericDesktop(GenericDesktopUsage::Y))
        })
        .expect("No Y axis");

    let (_, z_value) = result
        .iter()
        .find(|(data, _)| {
            data.global_state.usage_page == Some(UsagePage::GenericDesktop)
                && data.local_state.usage == Some(Usage::GenericDesktop(GenericDesktopUsage::Z))
        })
        .expect("No Z axis");

    let (_, rx_value) = result
        .iter()
        .find(|(data, _)| {
            data.global_state.usage_page == Some(UsagePage::GenericDesktop)
                && data.local_state.usage == Some(Usage::GenericDesktop(GenericDesktopUsage::Rx))
        })
        .expect("No Rx axis");

    let (_, ry_value) = result
        .iter()
        .find(|(data, _)| {
            data.global_state.usage_page == Some(UsagePage::GenericDesktop)
                && data.local_state.usage == Some(Usage::GenericDesktop(GenericDesktopUsage::Ry))
        })
        .expect("No Ry axis");

    let (_, rz_value) = result
        .iter()
        .find(|(data, _)| {
            data.global_state.usage_page == Some(UsagePage::GenericDesktop)
                && data.local_state.usage == Some(Usage::GenericDesktop(GenericDesktopUsage::Rz))
        })
        .expect("No Rz axis");

    let (_, slider_value) = result
        .iter()
        .find(|(data, _)| {
            data.global_state.usage_page == Some(UsagePage::GenericDesktop)
                && data.local_state.usage
                    == Some(Usage::GenericDesktop(GenericDesktopUsage::Slider))
        })
        .expect("No Slider axis");

    let (_, dial_value) = result
        .iter()
        .find(|(data, _)| {
            data.global_state.usage_page == Some(UsagePage::GenericDesktop)
                && data.local_state.usage == Some(Usage::GenericDesktop(GenericDesktopUsage::Dial))
        })
        .expect("No Slider axis");

    let target_buttons_value = 0;
    let target_x_value = 510;
    let target_y_value = 521;
    let target_rz_value = 512;
    let target_rx_value = 1023;
    let target_ry_value = 1023;
    let target_slider_value = 512;
    let target_dial_value = 629;
    let target_z_value = 35324;
    let target_pov_hat_value = 8;

    assert!(
        *buttons_value == target_buttons_value,
        "Buttons: {}, Target: {}",
        buttons_value,
        target_buttons_value
    );

    assert!(
        *x_value == target_x_value,
        "X Value: {}, Target: {}",
        x_value,
        target_x_value
    );

    assert!(
        *y_value == target_y_value,
        "Y Value: {}, Target: {}",
        y_value,
        target_y_value
    );

    assert!(
        *z_value == target_z_value,
        "Z Value: {}, Target: {}",
        z_value,
        target_z_value
    );

    assert!(
        *rx_value == target_rx_value,
        "Rx Value: {}, Target: {}",
        rx_value,
        target_rx_value
    );

    assert!(
        *ry_value == target_ry_value,
        "Ry Value: {}, Target: {}",
        ry_value,
        target_ry_value
    );

    assert!(
        *rz_value == target_rz_value,
        "Rz Value: {}, Target: {}",
        rz_value,
        target_rz_value
    );

    assert!(
        *slider_value == target_slider_value,
        "Slider Value: {}, Target: {}",
        slider_value,
        target_slider_value
    );

    assert!(
        *dial_value == target_dial_value,
        "Dial Value: {}, Target: {}",
        dial_value,
        target_dial_value
    );

    assert!(
        *pov_hat_value == target_pov_hat_value,
        "POV Hat Value: {}, Target: {}",
        pov_hat_value,
        target_pov_hat_value,
    );
}

#[test]
fn test_tfrp_input() {
    const TFRP_INPUT: [u8; 43] = [
        0xFF, 0x03, 0xFF, 0x03, 0xFB, 0x01, 0x14, 0x3B, 0x5E, 0x15, 0x35, 0x18, 0x04, 0x4F, 0x00,
        0x3C, 0xEA, 0x01, 0x70, 0x01, 0x50, 0x00, 0x80, 0x02, 0x00, 0x01, 0x00, 0x05, 0x50, 0x05,
        0x50, 0x00, 0x80, 0x01, 0x00, 0x01, 0x00, 0x04, 0x20, 0x04, 0x20, 0x00, 0x00,
    ];

    let report_desc = ReportDescriptor::new(TFRP_REPORT.iter().copied());
    let result = parse_input(&report_desc, TFRP_INPUT.iter().copied());

    let (_, x_value) = result
        .iter()
        .find(|(data, _)| {
            data.global_state.usage_page == Some(UsagePage::GenericDesktop)
                && data.local_state.usage == Some(Usage::GenericDesktop(GenericDesktopUsage::X))
        })
        .expect("No X axis");

    let (_, y_value) = result
        .iter()
        .find(|(data, _)| {
            data.global_state.usage_page == Some(UsagePage::GenericDesktop)
                && data.local_state.usage == Some(Usage::GenericDesktop(GenericDesktopUsage::Y))
        })
        .expect("No Y axis");

    let (_, rz_value) = result
        .iter()
        .find(|(data, _)| {
            data.global_state.usage_page == Some(UsagePage::GenericDesktop)
                && data.local_state.usage == Some(Usage::GenericDesktop(GenericDesktopUsage::Z))
        })
        .expect("No Z axis");

    let target_x_value = 1023;
    let target_y_value = 1023;
    let target_rz_value = 507;

    assert!(
        *x_value == target_x_value,
        "X Value: {}, Target: {}",
        x_value,
        target_x_value
    );
    assert!(
        *y_value == target_y_value,
        "Y Value: {}, Target: {}",
        y_value,
        target_y_value
    );
    assert!(
        *rz_value == target_rz_value,
        "Z Value: {}, Target: {}",
        rz_value,
        target_rz_value
    );
}
