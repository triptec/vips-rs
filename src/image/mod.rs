use ffi;
use ffi::{VipsSize, VipsKernel, VipsBandFormat, VipsCombineMode, VipsDirection, VipsInteresting, VipsIntent, VipsExtend, VipsArrayDouble};
use std::error::Error;
use std::os::raw::c_char;
use std::ptr::null;
use std::os::raw::c_void;
use std::ffi::CString;
use common::current_error;
use std::ptr::null_mut;
use std::marker::PhantomData;
use std::os::raw::c_int;
use ::VipsInterpolate;

use libffi::low::{types, ffi_type, ffi_cif, prep_cif_var, ffi_abi_FFI_DEFAULT_ABI, call, CodePtr};

#[derive(Default, Debug)]
pub struct VipsThumbnailOptions {
    pub height: Option<u32>,
    pub size: Option<VipsSize>,
    pub auto_rotate: Option<bool>,
    pub crop: Option<VipsInteresting>,
    pub linear: Option<bool>,
    pub import_profile: Option<String>,
    pub export_profile: Option<String>,
    pub intent: Option<VipsIntent>,
}

#[derive(Default)]
pub struct VipsEmbedOptions {
    pub extend: Option<VipsExtend>,
    pub background: Option<VipsArrayDouble>,
}

impl VipsThumbnailOptions {
    pub fn new() -> Self {
        VipsThumbnailOptions { height: None, size: None, auto_rotate: None, crop: None, linear: None, import_profile: None, export_profile: None, intent: None }
    }
}

pub struct VipsImage<'a> {
    pub c: *mut ffi::VipsImage,
    marker: PhantomData<&'a ()>,
}

impl<'a> Drop for VipsImage<'a> {
    fn drop(&mut self) {
        unsafe {
            ffi::g_object_unref(self.c as *mut c_void);
        }
    }
}

// callback used by gobjects
pub unsafe extern "C" fn image_postclose(ptr: *mut ffi::VipsImage, user_data: *mut c_void) {
    let b: Box<Box<[u8]>> = Box::from_raw(user_data as *mut Box<[u8]>);
    drop(b);
}

impl<'a> VipsImage<'a> {

    //
    // ─── CONSTRUCTORS ───────────────────────────────────────────────────────────────
    //

    pub fn new() -> Result<VipsImage<'a>, Box<Error>> {
        let c = unsafe { ffi::vips_image_new() };
        result(c)
    }

    pub fn new_memory() -> Result<VipsImage<'a>, Box<Error>> {
        let c = unsafe { ffi::vips_image_new_memory() };
        result(c)
    }

    pub fn from_file<S: Into<Vec<u8>>>(path: S) -> Result<VipsImage<'a>, Box<Error>> {
        let path = CString::new(path)?;
        let c = unsafe { ffi::vips_image_new_from_file(path.as_ptr(), null() as *const c_char) };
        result(c)
    }

    pub fn from_memory(buf: Vec<u8>, width: u32, height: u32, bands: u8, format: VipsBandFormat) -> Result<VipsImage<'a>, Box<Error>> {
        let b: Box<[_]> = buf.into_boxed_slice();
        let c = unsafe {
            ffi::vips_image_new_from_memory(
                b.as_ptr() as *const c_void,
                b.len(),
                width as i32,
                height as i32,
                bands as i32,
                format,
            )
        };

        let bb: Box<Box<_>> = Box::new(b);
        let raw: *mut c_void = Box::into_raw(bb) as *mut c_void;

        unsafe {
            let callback: unsafe extern "C" fn() = ::std::mem::transmute(image_postclose as *const ());
            ffi::g_signal_connect_data(
                c as *mut c_void, "postclose\0".as_ptr() as *const c_char,
                Some(callback),
                raw,
                None, ffi::GConnectFlags::G_CONNECT_AFTER);
        };

        result(c)
    }

    pub fn from_memory_reference(buf: &'a [u8], width: u32, height: u32, bands: u8, format: VipsBandFormat) -> Result<VipsImage, Box<Error>> {
        let c = unsafe {
            ffi::vips_image_new_from_memory(
                buf.as_ptr() as *const c_void,
                buf.len(),
                width as i32,
                height as i32,
                bands as i32,
                format,
            )
        };

        result(c)
    }

    // formatted
    pub fn from_buffer(buf: &'a [u8]) -> Result<VipsImage, Box<Error>> {
        let c = unsafe {
            ffi::vips_image_new_from_buffer(buf.as_ptr() as *const c_void, buf.len(), null(), null() as *const c_char)
        };

        result(c)
    }

    //
    // ─── DRAW ───────────────────────────────────────────────────────────────────────
    //

    pub fn draw_rect(&mut self, ink: &[f64], left: u32, top: u32, width: u32, height: u32) -> Result<(), Box<Error>> {
        let ret = unsafe {
            ffi::vips_draw_rect(
                self.c as *mut ffi::VipsImage,
                ink.as_ptr() as *mut f64,
                ink.len() as i32, left as i32,
                top as i32,
                width as i32,
                height as i32,
                null() as *const c_char)
        };
        result_draw(ret)
    }
    pub fn draw_rect1(&mut self, ink: f64, left: u32, top: u32, width: u32, height: u32) -> Result<(), Box<Error>> {
        let ret = unsafe {
            ffi::vips_draw_rect1(
                self.c as *mut ffi::VipsImage,
                ink,
                left as i32,
                top as i32,
                width as i32,
                height as i32,
                null() as *const c_char)
        };
        result_draw(ret)
    }
    pub fn draw_point(&mut self, ink: &[f64], x: i32, y: i32) -> Result<(), Box<Error>> {
        let ret = unsafe {
            ffi::vips_draw_point(
                self.c as *mut ffi::VipsImage,
                ink.as_ptr() as *mut f64,
                ink.len() as i32,
                x as i32,
                y as i32,
                null() as *const c_char)
        };
        result_draw(ret)
    }
    pub fn draw_point1(&mut self, ink: f64, x: i32, y: i32) -> Result<(), Box<Error>> {
        let ret = unsafe {
            ffi::vips_draw_point1(
                self.c as *mut ffi::VipsImage,
                ink,
                x as i32,
                y as i32,
                null() as *const c_char)
        };
        result_draw(ret)
    }
    pub fn draw_image(&mut self, img: &VipsImage, x: i32, y: i32, mode: VipsCombineMode) -> Result<(), Box<Error>> {
        let ret = unsafe {
            ffi::vips_draw_image(
                self.c as *mut ffi::VipsImage,
                img.c as *mut ffi::VipsImage,
                x as i32,
                y as i32,
                "mode\0".as_ptr(),
                mode,
                null() as *const c_char)
        };
        result_draw(ret)
    }
    pub fn draw_mask(&mut self, ink: &[f64], mask: &VipsImage, x: i32, y: i32) -> Result<(), Box<Error>> {
        let ret = unsafe {
            ffi::vips_draw_mask(
                self.c as *mut ffi::VipsImage,
                ink.as_ptr() as *mut f64,
                ink.len() as i32,
                mask.c as *mut ffi::VipsImage,
                x as i32,
                y as i32,
                null() as *const c_char)
        };
        result_draw(ret)
    }
    pub fn draw_mask1(&mut self, ink: f64, mask: &VipsImage, x: i32, y: i32) -> Result<(), Box<Error>> {
        let ret = unsafe {
            ffi::vips_draw_mask1(
                self.c as *mut ffi::VipsImage,
                ink,
                mask.c as *mut ffi::VipsImage,
                x as i32,
                y as i32,
                null() as *const c_char)
        };
        result_draw(ret)
    }
    pub fn draw_line(&mut self, ink: &[f64], x1: i32, y1: i32, x2: i32, y2: i32) -> Result<(), Box<Error>> {
        let ret = unsafe {
            ffi::vips_draw_line(
                self.c as *mut ffi::VipsImage,
                ink.as_ptr() as *mut f64,
                ink.len() as i32,
                x1 as i32,
                y1 as i32,
                x2 as i32,
                y2 as i32,
                null() as *const c_char)
        };
        result_draw(ret)
    }
    pub fn draw_line1(&mut self, ink: f64, x1: i32, y1: i32, x2: i32, y2: i32) -> Result<(), Box<Error>> {
        let ret = unsafe {
            ffi::vips_draw_line1(
                self.c as *mut ffi::VipsImage,
                ink,
                x1 as i32,
                y1 as i32,
                x2 as i32,
                y2 as i32,
                null() as *const c_char)
        };
        result_draw(ret)
    }
    pub fn draw_circle(&mut self, ink: &[f64], cx: i32, cy: i32, r: i32, fill: bool) -> Result<(), Box<Error>> {
        let ret = unsafe {
            ffi::vips_draw_circle(
                self.c as *mut ffi::VipsImage,
                ink.as_ptr() as *mut f64,
                ink.len() as i32,
                cx as i32,
                cy as i32,
                r as i32,
                "fill\0".as_ptr(),
                fill as i32,
                null() as *const c_char)
        };
        result_draw(ret)
    }
    pub fn draw_circle1(&mut self, ink: f64, cx: i32, cy: i32, r: i32, fill: bool) -> Result<(), Box<Error>> {
        let ret = unsafe {
            ffi::vips_draw_circle1(
                self.c as *mut ffi::VipsImage,
                ink,
                cx as i32,
                cy as i32,
                r as i32,
                "fill\0".as_ptr(),
                fill as i32,
                null() as *const c_char)
        };
        result_draw(ret)
    }
    pub fn draw_flood(&mut self, ink: &[f64], x: i32, y: i32) -> Result<(), Box<Error>> {
        let ret = unsafe {
            ffi::vips_draw_flood(
                self.c as *mut ffi::VipsImage,
                ink.as_ptr() as *mut f64,
                ink.len() as i32,
                x as i32,
                y as i32,
                null() as *const c_char)
        };
        result_draw(ret)
    }
    pub fn draw_flood1(&mut self, ink: f64, x: i32, y: i32) -> Result<(), Box<Error>> {
        let ret = unsafe {
            ffi::vips_draw_flood1(
                self.c as *mut ffi::VipsImage,
                ink,
                x as i32,
                y as i32,
                null() as *const c_char)
        };
        result_draw(ret)
    }
    pub fn draw_smudge(&mut self, left: u32, top: u32, width: u32, height: u32) -> Result<(), Box<Error>> {
        let ret = unsafe {
            ffi::vips_draw_smudge(
                self.c as *mut ffi::VipsImage,
                left as i32,
                top as i32,
                width as i32,
                height as i32,
                null() as *const c_char)
        };
        result_draw(ret)
    }

    //
    // ─── MOSAIC ─────────────────────────────────────────────────────────────────────
    //

    pub fn merge(&self, another: &VipsImage, direction: VipsDirection, dx: i32, dy: i32, mblend: Option<i32>) -> Result<VipsImage<'a>, Box<Error>> {
        let mut out_ptr: *mut ffi::VipsImage = null_mut();
        let ret = unsafe {
            ffi::vips_merge(
                self.c as *mut ffi::VipsImage,
                another.c as *mut ffi::VipsImage,
                &mut out_ptr,
                direction,
                dx,
                dy,
                "mblend\0".as_ptr(),
                mblend.unwrap_or(-1),
                null() as *const c_char)
        };
        result_with_ret(out_ptr, ret)
    }

    pub fn mosaic(&self, sec: &VipsImage, direction: VipsDirection, xref: i32, yref: i32, xsec: i32, ysec: i32, bandno: Option<i32>, hwindow: Option<i32>, harea: Option<i32>, mblend: Option<i32>) -> Result<VipsImage, Box<Error>> {
        let mut out_ptr: *mut ffi::VipsImage = null_mut();
        let ret = unsafe {
            ffi::vips_mosaic(
                self.c as *mut ffi::VipsImage,
                sec.c as *mut ffi::VipsImage,
                &mut out_ptr,
                direction,
                xref,
                yref,
                xsec,
                ysec,
                "bandno\0".as_ptr(),
                bandno.unwrap_or(0),
                "hwindow\0".as_ptr(),
                hwindow.unwrap_or(1),
                "harea\0".as_ptr(),
                harea.unwrap_or(1),
                "mblend\0".as_ptr(),
                mblend.unwrap_or(-1),
                null() as *const c_char)
        };
        result_with_ret(out_ptr, ret)
    }

    pub fn mosaic1(&self, sec: &VipsImage, direction: VipsDirection, xr1: i32, yr1: i32, xs1: i32, ys1: i32, xr2: i32, yr2: i32, xs2: i32, ys2: i32, search: Option<bool>, hwindow: Option<i32>, harea: Option<i32>, interpolate: Option<VipsInterpolate>, mblend: Option<i32>, bandno: Option<i32>) -> Result<VipsImage, Box<Error>> {
        let mut out_ptr: *mut ffi::VipsImage = null_mut();
        let ret = unsafe {
            match interpolate {
                Some(interpolate) => ffi::vips_mosaic1(
                    self.c,
                    sec.c,
                    &mut out_ptr,
                    direction,
                    xr1,
                    yr1,
                    xs1,
                    ys1,
                    xr2,
                    yr2,
                    xs2,
                    ys2,
                    "search\0".as_ptr(),
                    search.unwrap_or(false) as i32,
                    "hwindow\0".as_ptr(),
                    hwindow.unwrap_or(1),
                    "harea\0".as_ptr(),
                    harea.unwrap_or(1),
                    "interpolate\0".as_ptr(),
                    interpolate.c,
                    "mblend\0".as_ptr(),
                    mblend.unwrap_or(-1),
                    "bandno\0".as_ptr(),
                    bandno.unwrap_or(0),
                    null() as *const c_char),
                None => ffi::vips_mosaic1(
                    self.c as *mut ffi::VipsImage,
                    sec.c as *mut ffi::VipsImage,
                    &mut out_ptr,
                    direction,
                    xr1,
                    yr1,
                    xs1,
                    ys1,
                    xr2,
                    yr2,
                    xs2,
                    ys2,
                    "search\0".as_ptr(),
                    search.unwrap_or(false) as i32,
                    "hwindow\0".as_ptr(),
                    hwindow.unwrap_or(1),
                    "harea\0".as_ptr(),
                    harea.unwrap_or(1),
                    "mblend\0".as_ptr(),
                    mblend.unwrap_or(-1),
                    "bandno\0".as_ptr(),
                    bandno.unwrap_or(0),
                    null() as *const c_char)
            }
        };
        result_with_ret(out_ptr, ret)
    }

    pub fn match_(&self, sec: &VipsImage, xr1: i32, yr1: i32, xs1: i32, ys1: i32, xr2: i32, yr2: i32, xs2: i32, ys2: i32, search: Option<bool>, hwindow: Option<i32>, harea: Option<i32>, interpolate: Option<VipsInterpolate>) -> Result<VipsImage, Box<Error>> {
        let mut out_ptr: *mut ffi::VipsImage = null_mut();
        let ret = unsafe {
            match interpolate {
                Some(interpolate) => ffi::vips_match(
                    self.c as *mut ffi::VipsImage,
                    sec.c as *mut ffi::VipsImage,
                    &mut out_ptr,
                    xr1,
                    yr1,
                    xs1,
                    ys1,
                    xr2,
                    yr2,
                    xs2,
                    ys2,
                    "search".as_ptr(),
                    search.unwrap_or(false) as i32,
                    "hwindow".as_ptr(),
                    hwindow.unwrap_or(1),
                    "harea".as_ptr(),
                    harea.unwrap_or(1),
                    "interpolate".as_ptr(),
                    interpolate.c as *mut ffi::VipsInterpolate,
                    null() as *const c_char),
                None => ffi::vips_match(
                    self.c as *mut ffi::VipsImage,
                    sec.c as *mut ffi::VipsImage,
                    &mut out_ptr,
                    xr1,
                    yr1,
                    xs1,
                    ys1,
                    xr2,
                    yr2,
                    xs2,
                    ys2,
                    "search".as_ptr(),
                    search.unwrap_or(false) as i32,
                    "hwindow".as_ptr(),
                    hwindow.unwrap_or(1),
                    "harea".as_ptr(),
                    harea.unwrap_or(1),
                    null() as *const c_char)
            }
        };
        result_with_ret(out_ptr, ret)
    }

    pub fn globalbalance(&self, gamma: Option<f64>, int_output: Option<bool>) -> Result<VipsImage, Box<Error>> {
        let mut out_ptr: *mut ffi::VipsImage = null_mut();
        let ret = unsafe {
            ffi::vips_globalbalance(
                self.c as *mut ffi::VipsImage,
                &mut out_ptr,
                "gamma".as_ptr(),
                gamma.unwrap_or(1.6),
                "int_output".as_ptr(),
                int_output.unwrap_or(false) as i32,
                null() as *const c_char)
        };
        result_with_ret(out_ptr, ret)
    }

    pub fn remosaic(&self, old_str: &str, new_str: &str) -> Result<VipsImage, Box<Error>> {
        let old_str = CString::new(old_str)?;
        let new_str = CString::new(new_str)?;
        let mut out_ptr: *mut ffi::VipsImage = null_mut();
        let ret = unsafe {
            ffi::vips_remosaic(
                self.c as *mut ffi::VipsImage,
                &mut out_ptr,
                old_str.as_ptr(),
                new_str.as_ptr(),
                null() as *const c_char)
        };
        result_with_ret(out_ptr, ret)
    }


    //
    // ─── PROPERTIES ─────────────────────────────────────────────────────────────────
    //

    pub fn width(&self) -> u32 {
        unsafe { (*self.c).Xsize as u32 }
    }

    pub fn height(&self) -> u32 {
        unsafe { (*self.c).Ysize as u32 }
    }

    //
    // ─── RESIZE ─────────────────────────────────────────────────────────────────────
    //

    pub fn extract_area(&self, x: u32, y: u32, width: u32, height: u32) -> Result<VipsImage<'a>, Box<Error>> {
        let mut out_ptr: *mut ffi::VipsImage = null_mut();
        let mut out_dptr = &mut out_ptr;
        let mut in_ptr = self.c as *mut ffi::VipsImage;
        unsafe {
            let mut va_arguments = vec![
                &mut in_ptr as *mut _ as *mut c_void,
                &mut out_dptr as *mut _ as *mut c_void,
                &x as *const _ as *mut c_void,
                &y as *const _ as *mut c_void,
                &width as *const _ as *mut c_void,
                &height as *const _ as *mut c_void,
            ];
            let mut va_types: Vec<*mut ffi_type> = vec![&mut types::pointer,
                                                        &mut types::pointer,
                                                        &mut types::sint32,
                                                        &mut types::sint32,
                                                        &mut types::sint32,
                                                        &mut types::sint32,
            ];

            va_types.push(&mut types::pointer);
            //va_arguments.push("\0" as *const _ as *mut c_void);
            let end = null() as *const c_char;
            va_arguments.push(&end as *const _ as *mut c_void);
            //let end = CString::new("").unwrap();
            //va_arguments.push(&end as *const _ as *mut c_void);

            let mut cif: ffi_cif = Default::default();
            prep_cif_var(
                &mut cif,
                ffi_abi_FFI_DEFAULT_ABI,
                3,
                va_types.len(),
                &mut types::sint32,
                va_types.as_mut_ptr(),
            ).unwrap();
            let res: i32 = call(
                &mut cif,
                CodePtr(ffi::vips_extract_area as *mut _),
                va_arguments.as_mut_ptr(),
            );
            return result(*out_dptr);
        }
    }


    pub fn embed(&self, x: u32, y: u32, width: u32, height: u32, options: VipsEmbedOptions) -> Result<VipsImage<'a>, Box<Error>> {
        let mut out_ptr: *mut ffi::VipsImage = null_mut();
        let mut out_dptr = &mut out_ptr;
        let mut in_ptr = self.c as *mut ffi::VipsImage;
        unsafe {
            let mut va_arguments = vec![
                &mut in_ptr as *mut _ as *mut c_void,
                &mut out_dptr as *mut _ as *mut c_void,
                &x as *const _ as *mut c_void,
                &y as *const _ as *mut c_void,
                &width as *const _ as *mut c_void,
                &height as *const _ as *mut c_void,
            ];
            let mut va_types: Vec<*mut ffi_type> = vec![&mut types::pointer,
                                                        &mut types::pointer,
                                                        &mut types::sint32,
                                                        &mut types::sint32,
                                                        &mut types::sint32,
                                                        &mut types::sint32,
            ];

            let extend_attr = "extend\0";
            let extend_attr_ptr_void = &extend_attr.as_ptr() as *const _ as *mut c_void;

            if options.extend.is_some() {
                va_types.push(&mut types::pointer);
                va_arguments.push(extend_attr_ptr_void);

                va_types.push(&mut types::uint32);
                if let Some(ref v) = options.extend {
                    va_arguments.push(v as *const _ as *mut c_void);
                }
            }

            va_types.push(&mut types::pointer);
            //va_arguments.push("\0" as *const _ as *mut c_void);
            let end = null() as *const c_char;
            va_arguments.push(&end as *const _ as *mut c_void);
            //let end = CString::new("").unwrap();
            //va_arguments.push(&end as *const _ as *mut c_void);

            let mut cif: ffi_cif = Default::default();
            prep_cif_var(
                &mut cif,
                ffi_abi_FFI_DEFAULT_ABI,
                3,
                va_types.len(),
                &mut types::sint32,
                va_types.as_mut_ptr(),
            ).unwrap();
            let res: i32 = call(
                &mut cif,
                CodePtr(ffi::vips_embed as *mut _),
                va_arguments.as_mut_ptr(),
            );
            return result(*out_dptr);
        }
    }

    pub fn thumbnail(&self, width: u32, options: VipsThumbnailOptions) -> Result<VipsImage<'a>, Box<Error>> {
        dbg!(&options);
        let mut out_ptr: *mut ffi::VipsImage = null_mut();
        let mut out_dptr = &mut out_ptr;
        let mut in_ptr = self.c as *mut ffi::VipsImage;
        /*
        unsafe {
            //ffi::vips_black(out_dptr, 5, 5, null() as *const c_char);
            ffi::vips_thumbnail_image(in_ptr, out_dptr, 200, null() as *const c_char);
            return result(*out_dptr);
        }
        */


        unsafe {
            let mut va_arguments = vec![
                &mut in_ptr as *mut _ as *mut c_void,
                &mut out_dptr as *mut _ as *mut c_void,
                &width as *const _ as *mut c_void,
            ];
            let mut va_types: Vec<*mut ffi_type> = vec![&mut types::pointer,
                                                        &mut types::pointer,
                                                        &mut types::sint32,
            ];

            let height_attr = "height\0";
            let height_attr_ptr_void = &height_attr.as_ptr() as *const _ as *mut c_void;

            if options.height.is_some() {
                va_types.push(&mut types::pointer);
                va_arguments.push(height_attr_ptr_void);

                va_types.push(&mut types::sint32);
                if let Some(ref v) = options.height {
                    va_arguments.push(v as *const _ as *mut c_void);
                }
            }

            let size_attr = "size\0";
            let size_attr_ptr_void = &size_attr.as_ptr() as *const _ as *mut c_void;
            if options.size.is_some() {
                va_types.push(&mut types::pointer);
                va_arguments.push(size_attr_ptr_void);

                va_types.push(&mut types::uint32);
                if let Some(ref v) = options.size {
                    va_arguments.push(v as *const _ as *mut c_void);
                }
            }

            let auto_rotate_attr = "auto_rotate\0";
            let auto_rotate_attr_ptr_void = &auto_rotate_attr.as_ptr() as *const _ as *mut c_void;
            if options.auto_rotate.is_some() {
                va_types.push(&mut types::pointer);
                va_arguments.push(auto_rotate_attr_ptr_void);

                va_types.push(&mut types::uint8);
                if let Some(ref v) = options.auto_rotate {
                    va_arguments.push(v as *const _ as *mut c_void);
                }
            }

            let crop_attr = "crop\0";
            let crop_attr_ptr_void = &crop_attr.as_ptr() as *const _ as *mut c_void;
            if options.crop.is_some() {
                va_types.push(&mut types::pointer);
                va_arguments.push(crop_attr_ptr_void);

                va_types.push(&mut types::uint32);
                if let Some(ref v) = options.crop {
                    va_arguments.push(v as *const _ as *mut c_void);
                }
            }

            let linear_attr = "linear\0";
            let linear_attr_ptr_void = &linear_attr.as_ptr() as *const _ as *mut c_void;
            if options.linear.is_some() {
                va_types.push(&mut types::pointer);
                va_arguments.push(linear_attr_ptr_void);

                va_types.push(&mut types::uint8);
                if let Some(ref v) = options.linear {
                    va_arguments.push(v as *const _ as *mut c_void);
                }
            }


            // Written not tested
            let import_profile_attr = "import_profile\0";
            let import_profile_attr_ptr_void = &import_profile_attr.as_ptr() as *const _ as *mut c_void;
            let import_profile_value = match options.import_profile {
                Some(v) => Some(CString::new(v).unwrap()),
                None => None
            };
            if import_profile_value.is_some() {
                va_types.push(&mut types::pointer);
                va_arguments.push(import_profile_attr_ptr_void);

                va_types.push(&mut types::pointer);

                va_arguments.push(&mut import_profile_value.unwrap().as_ptr() as *mut _ as *mut c_void);
            }

            // Written not tested
            let export_profile_attr = "export_profile\0";
            let export_profile_attr_ptr_void = &export_profile_attr.as_ptr() as *const _ as *mut c_void;
            let export_profile_value = match options.export_profile {
                Some(v) => Some(CString::new(v).unwrap()),
                None => None
            };
            if export_profile_value.is_some() {
                va_types.push(&mut types::pointer);
                va_arguments.push(export_profile_attr_ptr_void);

                va_types.push(&mut types::pointer);

                va_arguments.push(&mut export_profile_value.unwrap().as_ptr() as *mut _ as *mut c_void);
            }

            let intent_attr = "intent\0";
            let intent_attr_ptr_void = &intent_attr.as_ptr() as *const _ as *mut c_void;
            if options.intent.is_some() {
                va_types.push(&mut types::pointer);
                va_arguments.push(intent_attr_ptr_void);

                va_types.push(&mut types::uint32);
                if let Some(ref v) = options.intent {
                    va_arguments.push(v as *const _ as *mut c_void);
                }
            }

            va_types.push(&mut types::pointer);
            let end = null() as *const c_char;
            va_arguments.push(&end as *const _ as *mut c_void);

            let mut cif: ffi_cif = Default::default();
            prep_cif_var(
                &mut cif,
                ffi_abi_FFI_DEFAULT_ABI,
                3,
                va_types.len(),
                &mut types::sint32,
                va_types.as_mut_ptr(),
            ).unwrap();
            let res: i32 = call(
                &mut cif,
                CodePtr(ffi::vips_thumbnail_image as *mut _),
                va_arguments.as_mut_ptr(),
            );
            return result(*out_dptr);
        };
    }

    pub fn copy(&self) -> Result<VipsImage<'a>, Box<Error>> {
        let mut out_ptr: *mut ffi::VipsImage = null_mut();
        let ret = unsafe {
            ffi::vips_copy(
                self.c as *mut ffi::VipsImage,
                &mut out_ptr,
                null() as *const c_char,
            )
        };
        result(out_ptr)
    }

    pub fn smartcrop(&self, width: u32, height: u32, interesting: VipsInteresting) -> Result<VipsImage<'a>, Box<Error>> {
        let mut out_ptr: *mut ffi::VipsImage = null_mut();
        let ret = unsafe {
            ffi::vips_smartcrop(
                self.c as *mut ffi::VipsImage,
                &mut out_ptr,
                width as i32,
                height as i32,
                "interesting\0".as_ptr(),
                interesting,
                null() as *const c_char,
            )
        };
        result(out_ptr)
    }

    pub fn hist_find(&self) -> Result<VipsImage<'a>, Box<Error>> {
        let mut out_ptr: *mut ffi::VipsImage = null_mut();
        let ret = unsafe {
            ffi::vips_hist_find(
                self.c as *mut ffi::VipsImage,
                &mut out_ptr,
                null() as *const c_char,
            )
        };
        result(out_ptr)
    }

    pub fn hist_entropy(&self) -> Result<f64, Box<Error>> {
        let mut out: f64 = 0.0;
        let ret = unsafe {
            ffi::vips_hist_entropy(
                self.c as *mut ffi::VipsImage,
                &mut out,
                null() as *const c_char,
            )
        };
        if ret == 0 {
            Ok(out)
        } else {
            Err(current_error().into())
        }
    }

    // default: block shrink + lanczos3
    pub fn resize(&self, scale: f64, vscale: Option<f64>, kernel: Option<VipsKernel>) -> Result<VipsImage<'a>, Box<Error>> {
        let mut out_ptr: *mut ffi::VipsImage = null_mut();
        let ret = unsafe {
            ffi::vips_resize(
                self.c as *mut ffi::VipsImage,
                &mut out_ptr,
                scale,
                "vscale\0".as_ptr(),
                vscale.unwrap_or(scale),
                "kernel\0".as_ptr(),
                kernel.unwrap_or(VipsKernel::VIPS_KERNEL_LANCZOS3),
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    fn resize_to_size(&self, width: u32, height: Option<u32>, kernel: Option<VipsKernel>) -> Result<VipsImage, Box<Error>> {
        self.resize(
            width as f64 / self.width() as f64,
            height.map(|h| h as f64 / self.height() as f64),
            kernel,
        )
    }

    // low-level
    // default: 2 * 1D lanczos3 (not recommended for shrink factor > 3)
    // or other kernels
    fn reduce(&self, hshrink: f64, vshrink: f64, kernel: Option<VipsKernel>, centre: Option<bool>) -> VipsImage {
        unimplemented!();
//        unsafe {
//            ffi::vips_reduce(self.c, , )
//        }
    }

    fn shrink(&self) -> VipsImage { // simple average of nxn -> 1/n size
        unimplemented!();
    }

    //
    // ─── IO ─────────────────────────────────────────────────────────────────────────
    //

    fn jpegsave<S: Into<Vec<u8>>>(&mut self, path: S) -> Result<(), Box<Error>> {
        let path = CString::new(path)?;
        let ret = unsafe { ffi::vips_jpegsave(self.c as *mut ffi::VipsImage, path.as_ptr(), null() as *const c_char) };
        match ret {
            0 => Ok(()),
            _ => Err(current_error().into()),
        }
    }

    pub fn write_to_file<S: Into<Vec<u8>>>(&self, path: S) -> Result<(), Box<Error>> {
        let path = CString::new(path)?;
        let ret = unsafe { ffi::vips_image_write_to_file(self.c as *mut ffi::VipsImage, path.as_ptr(), null() as *const c_char) };
        match ret {
            0 => Ok(()),
            _ => Err(current_error().into()),
        }
    }

    pub fn write_to_buffer<S: Into<Vec<u8>>>(&self, suffix: S) -> Result<(Vec<u8>), Box<Error>> {
        //let mut memory std::ffi::c_void;
        let mut buf: *mut u8 = null_mut();

        let mut_ref: &mut *mut u8 = &mut buf;

        let raw_ptr: *mut *mut u8 = mut_ref as *mut *mut _;

        let void_cast: *mut *mut c_void = raw_ptr as *mut *mut c_void;

        //let mut memory: Vec<u8> = Vec::new();
        let mut result_size: usize = 0;

        let suffix = CString::new(suffix)?;
        let ret = unsafe { ffi::vips_image_write_to_buffer(self.c as *mut ffi::VipsImage, suffix.as_ptr(), void_cast, &mut result_size as *mut usize, null() as *const c_char) };
        //dbg!(buf);
        let slice = unsafe { ::std::slice::from_raw_parts_mut(&mut *buf, result_size) };
        let boxed_slice: Box<[u8]> = unsafe { Box::from_raw(slice) };
        let vec = boxed_slice.into_vec();
        Ok(vec)
    }

    pub fn magicksave_buffer<S: Into<Vec<u8>>>(&self, q: u32, format: S) -> Result<(Vec<u8>), Box<Error>> {
        //let mut memory std::ffi::c_void;
        let mut buf: *mut u8 = null_mut();

        let mut_ref: &mut *mut u8 = &mut buf;

        let raw_ptr: *mut *mut u8 = mut_ref as *mut *mut _;

        let void_cast: *mut *mut c_void = raw_ptr as *mut *mut c_void;

        //let mut memory: Vec<u8> = Vec::new();
        let mut result_size: usize = 0;

        let format = CString::new(format)?;
        let ret = unsafe {
            ffi::vips_magicksave_buffer(
                self.c as *mut ffi::VipsImage,
                void_cast,
                &mut result_size as *mut usize,
                "quality\0".as_ptr(),
                q,
                "format\0".as_ptr(),
                format.as_ptr(),
                null() as *const c_char
            )
        };
        let slice = unsafe { ::std::slice::from_raw_parts_mut(&mut *buf, result_size) };
        let boxed_slice: Box<[u8]> = unsafe { Box::from_raw(slice) };
        let vec = boxed_slice.into_vec();
        Ok(vec)
    }

    pub fn jpegsave_buffer(&self, q: u32) -> Result<(Vec<u8>), Box<Error>> {
        //let mut memory std::ffi::c_void;
        let mut buf: *mut u8 = null_mut();

        let mut_ref: &mut *mut u8 = &mut buf;

        let raw_ptr: *mut *mut u8 = mut_ref as *mut *mut _;

        let void_cast: *mut *mut c_void = raw_ptr as *mut *mut c_void;

        //let mut memory: Vec<u8> = Vec::new();
        let mut result_size: usize = 0;

        let ret = unsafe {
            ffi::vips_jpegsave_buffer(
                self.c as *mut ffi::VipsImage,
                void_cast,
                &mut result_size as *mut usize,
                "Q\0".as_ptr(),
                q,
                null() as *const c_char
            )
        };
        let slice = unsafe { ::std::slice::from_raw_parts_mut(&mut *buf, result_size) };
        let boxed_slice: Box<[u8]> = unsafe { Box::from_raw(slice) };
        let vec = boxed_slice.into_vec();
        Ok(vec)
    }

    pub fn webpsave_buffer(&self, q: u32, lossless: bool) -> Result<(Vec<u8>), Box<Error>> {
        //let mut memory std::ffi::c_void;
        let mut buf: *mut u8 = null_mut();

        let mut_ref: &mut *mut u8 = &mut buf;

        let raw_ptr: *mut *mut u8 = mut_ref as *mut *mut _;

        let void_cast: *mut *mut c_void = raw_ptr as *mut *mut c_void;

        //let mut memory: Vec<u8> = Vec::new();
        let mut result_size: usize = 0;

        let ret = unsafe {
            ffi::vips_webpsave_buffer(
                self.c as *mut ffi::VipsImage,
                void_cast,
                &mut result_size as *mut usize,
                "Q\0".as_ptr(),
                q,
                "lossless\0".as_ptr(),
                lossless as c_int,
                null() as *const c_char
            )
        };
        let slice = unsafe { ::std::slice::from_raw_parts_mut(&mut *buf, result_size) };
        let boxed_slice: Box<[u8]> = unsafe { Box::from_raw(slice) };
        let vec = boxed_slice.into_vec();
        Ok(vec)
    }

    pub fn pngsave_buffer(&self) -> Result<(Vec<u8>), Box<Error>> {
        //let mut memory std::ffi::c_void;
        let mut buf: *mut u8 = null_mut();

        let mut_ref: &mut *mut u8 = &mut buf;

        let raw_ptr: *mut *mut u8 = mut_ref as *mut *mut _;

        let void_cast: *mut *mut c_void = raw_ptr as *mut *mut c_void;

        //let mut memory: Vec<u8> = Vec::new();
        let mut result_size: usize = 0;

        let ret = unsafe {
            ffi::vips_pngsave_buffer(
                self.c as *mut ffi::VipsImage,
                void_cast,
                &mut result_size as *mut usize,
                null() as *const c_char
            )
        };
        let slice = unsafe { ::std::slice::from_raw_parts_mut(&mut *buf, result_size) };
        let boxed_slice: Box<[u8]> = unsafe { Box::from_raw(slice) };
        let vec = boxed_slice.into_vec();
        Ok(vec)
    }

    //
    // ─── CONVERT ────────────────────────────────────────────────────────────────────
    //

    pub fn to_vec(&self) -> Vec<u8> {
        unsafe {
            let mut result_size: usize = 0;
            let memory: *mut u8 = ffi::vips_image_write_to_memory(self.c as *mut ffi::VipsImage, &mut result_size as *mut usize) as *mut u8;
            let slice = ::std::slice::from_raw_parts_mut(memory, result_size);
            let boxed_slice: Box<[u8]> = Box::from_raw(slice);
            let vec = boxed_slice.into_vec();
            vec
        }
    }
}

fn result<'a>(ptr: *mut ffi::VipsImage) -> Result<VipsImage<'a>, Box<Error>> {
    if ptr.is_null() {
        Err(current_error().into())
    } else {
        Ok(VipsImage { c: ptr, marker: PhantomData })
    }
}

fn result_with_ret<'a>(ptr: *mut ffi::VipsImage, ret: c_int) -> Result<VipsImage<'a>, Box<Error>> {
    if ret == 0 {
        Ok(VipsImage { c: ptr, marker: PhantomData })
    } else {
        Err(current_error().into())
    }
}

fn result_draw(ret: ::std::os::raw::c_int) -> Result<(), Box<Error>> {
    match ret {
        0 => Ok(()),
        -1 => Err(current_error().into()),
        _ => Err("Unknown error from libvips".into())
    }
}
