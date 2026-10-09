//! تولید QR Code برای URL اتصال

use qrcode::QrCode;
use qrcode::render::svg;

/// تولید QR Code به صورت رشته‌ی SVG
/// 
/// SVG یه فرمت برداریه که تو HTML مستقیم قابل embed هست.
/// چرا SVG نه PNG؟ چون حجمش کمه و هر اندازه‌ای رندر میشه.
pub fn generate_qr_svg(data: &str) -> Result<String, String> {
    let code = QrCode::new(data)
        .map_err(|e| format!("خطا در تولید QR: {}", e))?;
    
    let svg_string = code
        .render::<svg::Color>()
        .min_dimensions(256, 256)
        .dark_color(svg::Color("#1a1a2e"))
        .light_color(svg::Color("#ffffff"))
        .build();
    
    Ok(svg_string)
}