extern crate vips;

use vips::*;

fn crop_file() {
    let thumbnail = {
        let img: VipsImage = VipsImage::from_file("kodim01.png").unwrap();
        let mut options = VipsThumbnailOptions::new();
        options.height = Some(234);
        options.crop = Some(VipsInteresting::VIPS_INTERESTING_CENTRE);
        img.thumbnail(234, options).unwrap()
    };

    thumbnail.write_to_file("kodim01_234x234.png").unwrap();
}

fn resize_file() {
    let thumbnail = {
        let img: VipsImage = VipsImage::from_file("kodim01.png").unwrap();
        let mut options = VipsThumbnailOptions::new();
        options.height = Some(234);
        options.size = Some(VipsSize::VIPS_SIZE_FORCE);
        img.thumbnail(123, options).unwrap()
    };

    thumbnail.write_to_file("kodim01_123x234.png").unwrap();
}

fn resize_mem() {
    let pixels = vec![0; 256 * 256 * 3];
    let thumbnail = {
        let img: VipsImage = VipsImage::from_memory(pixels, 256, 256, 3, VipsBandFormat::VIPS_FORMAT_UCHAR).unwrap();
        let mut options = VipsThumbnailOptions::new();
        options.height = Some(123);
        options.size = Some(VipsSize::VIPS_SIZE_FORCE);
        img.thumbnail(234, options).unwrap()
    };
    thumbnail.write_to_file("black_mem_234_123.png").unwrap();
}

fn resize_mem_ref() {
    let pixels = vec![0; 256 * 256 * 3];
    let thumbnail = {
        let img: VipsImage = VipsImage::from_memory_reference(&pixels, 256, 256, 3, VipsBandFormat::VIPS_FORMAT_UCHAR).unwrap();
        let mut options = VipsThumbnailOptions::new();
        options.height = Some(123);
        options.size = Some(VipsSize::VIPS_SIZE_FORCE);
        img.thumbnail(234, options).unwrap()
    };
    thumbnail.write_to_file("black_ref_234x123.png").unwrap();
}

fn main() {
    let path = std::env::current_dir().unwrap();
    println!("The current directory is {}", path.display());
    let _instance = VipsInstance::new("app_test", true).unwrap();
    crop_file();
    resize_file();
    resize_mem();
    resize_mem_ref();
}
