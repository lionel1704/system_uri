// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

#![allow(
    non_snake_case, missing_docs, unsafe_code, unused_results, trivial_casts, trivial_numeric_casts,
    unused, unused_qualifications
)]
extern crate jni;
extern crate system_uri;
#[macro_use]
extern crate ffi_utils;
#[macro_use]
extern crate unwrap;

use ffi_utils::*;
use jni::errors::{Error as JniError, ErrorKind};
use jni::objects::{GlobalRef, JClass, JObject, JString};
use jni::strings::JNIStr;
use jni::sys::{jbyte, jbyteArray, jint, jlong, jobject, jsize};
use jni::{JNIEnv, JavaVM};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::{cmp, mem, slice};


static mut JVM: Option<JavaVM> = None;

pub type JniResult<T> = Result<T, JniError>;

/// Trait for conversion of Rust value to Java value.
pub trait ToJava<'a, T: Sized + 'a> {
    /// Converts Rust value to Java value
    fn to_java(&self, env: &'a JNIEnv) -> JniResult<T>;
}

/// Trait for conversion of Java value to Rust value.
pub trait FromJava<T> {
    /// Converts Java value to Rust value
    fn from_java(env: &JNIEnv, input: T) -> JniResult<Self>
    where
        Self: Sized;
}

impl<'a, 'b> ToJava<'a, JObject<'a>> for &'b [i32] {
    fn to_java(&self, env: &'a JNIEnv) -> JniResult<JObject<'a>> {
        let output = env.new_int_array(self.len() as jsize)?;
        env.set_int_array_region(output, 0, self)?;
        Ok(JObject::from(output as jobject))
    }
}

impl<'a> FromJava<JObject<'a>> for FfiResult {
    fn from_java(env: &JNIEnv, input: JObject) -> Result<Self, JniError> {
        let error_code = env.get_field(input, "errorCode", "I")?.i()? as i32;
        let description = env.get_field(input, "description", "Ljava/lang/String;")?
            .l()?
            .into();
        let description = <*mut _>::from_java(env, description)?;
        Ok(FfiResult {
            error_code,
            description,
        })
    }
}
impl<'a> ToJava<'a, JObject<'a>> for FfiResult {
    fn to_java(&self, env: &'a JNIEnv) -> Result<JObject<'a>, JniError> {
        let output = env.new_object(
            "net/maidsafe/safe_app/FfiResult",
            "()V",
            &[],
        )?;
        env.set_field(
            output,
            "errorCode",
            "I",
            self.error_code.to_java(env)?.into(),
        )?;
        if !self.description.is_null() {
            let description: JObject = self.description.to_java(env)?.into();
            env.set_field(
                output,
                "description",
                "Ljava/lang/String;",
                description.into(),
            )?;
        }
        Ok(output)
    }
}

impl<'a> FromJava<JString<'a>> for CString {
    fn from_java(env: &JNIEnv, input: JString) -> JniResult<Self> {
        let tmp: &CStr = &*unwrap!(env.get_string(input));
        Ok(tmp.to_owned())
    }
}

impl<'a> FromJava<JString<'a>> for *mut c_char {
    fn from_java(env: &JNIEnv, input: JString) -> JniResult<Self> {
        Ok(<*const _>::from_java(env, input)? as *mut _)
    }
}

impl<'a> ToJava<'a, JString<'a>> for *mut c_char {
    fn to_java(&self, env: &'a JNIEnv) -> JniResult<JString<'a>> {
        Ok((*self as *const c_char).to_java(env)?)
    }
}


impl<'a, 'b> ToJava<'a, JObject<'a>> for &'b [u8] {
    fn to_java(&self, env: &'a JNIEnv) -> JniResult<JObject<'a>> {
        let output = env.new_byte_array(self.len() as jsize)?;
        env.set_byte_array_region(output, 0, unsafe {
            slice::from_raw_parts(self.as_ptr() as *const i8, self.len())
        })?;
        Ok(JObject::from(output as jobject))
    }
}

impl<'a> FromJava<JObject<'a>> for Vec<u8> {
    fn from_java(env: &JNIEnv, input: JObject) -> JniResult<Self> {
        let input = input.into_inner() as jbyteArray;
        Ok(env.convert_byte_array(input)?)
    }
}

// This is called when `loadLibrary` is called on the Java side.
#[no_mangle]
pub unsafe extern "C" fn JNI_OnLoad(vm: *mut jni::sys::JavaVM, _reserved: *mut c_void) -> jint {
    JVM = Some(unwrap!(JavaVM::from_raw(vm)));
    jni::sys::JNI_VERSION_1_4
}

/// Converts `user_data` back into a Java callback object
pub unsafe fn convert_cb_from_java(env: &JNIEnv, ctx: *mut c_void) -> GlobalRef {
    GlobalRef::from_raw(unwrap!(env.get_java_vm()), ctx as jobject)
}

/// Unwraps the results and checks for Java exceptions.
/// Required for exceptions pass-through (simplifies debugging).
#[macro_export]
macro_rules! jni_unwrap {
    ($res:expr) => {{
        let res: Result<_, JniError> = $res;
        if let Err(JniError(ErrorKind::JavaException, _)) = res {
            return;
        } else {
            res.unwrap()
        }
    }};
}

/// Generates a `user_data` context containing a reference to a single or several Java callbacks
#[macro_export]
macro_rules! gen_ctx {
    ($env:ident, $cb:ident) => {
        {
            let ctx = $env.new_global_ref($cb).unwrap();
            $env.delete_local_ref($cb).unwrap();
            let ptr = *ctx.as_obj() as *mut c_void;
            mem::forget(ctx);
            ptr
        }
    };

    ($env:ident, $cb0:ident, $($cb_rest:ident),+ ) => {
        {
            let ctx = [
                Some($env.new_global_ref($cb0).unwrap()),
                $(
                    Some($env.new_global_ref($cb_rest).unwrap()),
                )+
            ];
            let ctx = Box::into_raw(Box::new(ctx)) as *mut c_void;
            $env.delete_local_ref($cb0).unwrap();
            $(
                $env.delete_local_ref($cb_rest).unwrap();
            )+
            ctx
        }
    }
}



include!("../../bindings/java/system_uri/jni.rs");
