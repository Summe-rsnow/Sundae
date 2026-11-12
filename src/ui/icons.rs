use nsh_core::prelude::*;
use std::sync::LazyLock;

// Import standalone SVG files from assets/icons/
const SUNDAE_SVG: &str = include_str!("../../assets/icons/sundae.svg");
const SEARCH_SVG: &str = include_str!("../../assets/icons/search.svg");
const SEARCH_LIGHT_SVG: &str = include_str!("../../assets/icons/search_light.svg");
const PACKAGE_SVG: &str = include_str!("../../assets/icons/package.svg");
const REFRESH_SVG: &str = include_str!("../../assets/icons/refresh.svg");
const WAREHOUSE_SVG: &str = include_str!("../../assets/icons/warehouse.svg");
const SETTINGS_SVG: &str = include_str!("../../assets/icons/settings.svg");
const TERMINAL_SVG: &str = include_str!("../../assets/icons/terminal.svg");
const FOLDER_SVG: &str = include_str!("../../assets/icons/folder.svg");
const FILE_TEXT_SVG: &str = include_str!("../../assets/icons/file_text.svg");
const LOCK_SVG: &str = include_str!("../../assets/icons/lock.svg");
const UNLOCK_SVG: &str = include_str!("../../assets/icons/unlock.svg");
const TRASH_SVG: &str = include_str!("../../assets/icons/trash.svg");
const CHECK_SVG: &str = include_str!("../../assets/icons/check.svg");
const X_SVG: &str = include_str!("../../assets/icons/x.svg");
const ARROW_RIGHT_SVG: &str = include_str!("../../assets/icons/arrow_right.svg");
const DOWNLOAD_SVG: &str = include_str!("../../assets/icons/download.svg");
const GLOBE_SVG: &str = include_str!("../../assets/icons/globe.svg");
const ZAP_SVG: &str = include_str!("../../assets/icons/zap.svg");
const GIT_BRANCH_SVG: &str = include_str!("../../assets/icons/git_branch.svg");
const SLIDERS_SVG: &str = include_str!("../../assets/icons/sliders.svg");
const PLUS_SVG: &str = include_str!("../../assets/icons/plus.svg");
const SHIELD_CHECK_SVG: &str = include_str!("../../assets/icons/shield_check.svg");
const ALERT_CIRCLE_SVG: &str = include_str!("../../assets/icons/alert_circle.svg");

fn to_data_uri(svg_content: &str) -> String {
    let mut encoded = String::from("data:image/svg+xml;utf8,");
    for b in svg_content.bytes() {
        match b {
            b' ' | b'"' | b'#' | b'%' | b'<' | b'>' => {
                encoded.push('%');
                encoded.push_str(&format!("{:02X}", b));
            }
            _ => encoded.push(b as char),
        }
    }
    encoded
}

macro_rules! define_icon {
    ($fn_name:ident, $const_svg:ident) => {
        pub fn $fn_name(size: f32) -> Svg {
            static URI: LazyLock<String> = LazyLock::new(|| to_data_uri($const_svg));
            svg(URI.as_str()).w(px(size)).h(px(size))
        }
    };
}

define_icon!(sundae_logo, SUNDAE_SVG);
define_icon!(search_icon, SEARCH_SVG);
define_icon!(search_icon_light, SEARCH_LIGHT_SVG);
define_icon!(package_icon, PACKAGE_SVG);
define_icon!(refresh_icon, REFRESH_SVG);
define_icon!(bucket_icon, WAREHOUSE_SVG);
define_icon!(settings_icon, SETTINGS_SVG);
define_icon!(terminal_icon, TERMINAL_SVG);
define_icon!(folder_icon, FOLDER_SVG);
define_icon!(file_text_icon, FILE_TEXT_SVG);
define_icon!(lock_icon, LOCK_SVG);
define_icon!(unlock_icon, UNLOCK_SVG);
define_icon!(trash_icon, TRASH_SVG);
define_icon!(check_icon, CHECK_SVG);
define_icon!(x_icon, X_SVG);
define_icon!(arrow_right_icon, ARROW_RIGHT_SVG);
define_icon!(download_icon, DOWNLOAD_SVG);
define_icon!(globe_icon, GLOBE_SVG);
define_icon!(zap_icon, ZAP_SVG);
define_icon!(git_branch_icon, GIT_BRANCH_SVG);
define_icon!(sliders_icon, SLIDERS_SVG);
define_icon!(plus_icon, PLUS_SVG);
define_icon!(shield_check_icon, SHIELD_CHECK_SVG);
define_icon!(alert_circle_icon, ALERT_CIRCLE_SVG);
