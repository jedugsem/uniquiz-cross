use jni::objects::JObject;
use jni::objects::JValue;
use jni::sys::jobject;
use jni::{AttachGuard, JavaVM};
use log::info;
pub fn get_context() -> jobject {
    ndk_context::android_context().context().cast()
}

pub fn get_activity() -> JObject<'static> {
    unsafe { JObject::from_raw(get_context()) } // unsafe { JObject::from_raw(context()) }
}

fn has_permissions(permissions: &[&str]) -> bool {
    let ctx = ndk_context::android_context();

    let vm = get_vm(&ctx);
    let mut env = vm.attach_current_thread().unwrap();

    for &permission in permissions {
        let perm_jstring = env.new_string(permission).unwrap();
        let permission_status = env
            .call_method(
                get_activity(),
                "checkSelfPermission",
                "(Ljava/lang/String;)I",
                &[(&perm_jstring).into()],
            )
            .unwrap()
            .i()
            .unwrap();

        if permission_status != 0 {
            return false;
        }
    }

    true
}

pub fn get_permission(permissions: &[&str]) {
    let ctx = ndk_context::android_context();
    let vm = get_vm(&ctx);
    let mut env = vm.attach_current_thread().unwrap();

    let string_class = env.find_class("java/lang/String").unwrap();
    let default_string = env.new_string("").unwrap();
    let mut permissions_array = env
        .new_object_array(permissions.len() as i32, string_class, default_string)
        .unwrap();

    for (i, &permission) in permissions.iter().enumerate() {
        let java_permission = env.new_string(permission).unwrap();
        env.set_object_array_element(&permissions_array, i as i32, java_permission)
            .unwrap(); // &mut permissions_array
    }

    if !has_permissions(permissions) {
        env.call_method(
            get_activity(),
            "requestPermissions",
            "([Ljava/lang/String;I)V",
            &[(&permissions_array).into(), 0.into()],
        )
        .unwrap();
    }

    info!("permissions: {:?}", has_permissions(permissions));
    // todo: handle case where permission is rejected
}
//
// some jni syntax hints
//
// L - class, for example: Ljava/lang/String;
// primitives: Z - boolean, I - integer, V - void
// for example, Rust signature fn get_text(flag: bool) -> String
// will become "(Z)Ljava/lang/String;"
//
// in find_class . is replaced by $
// docs: android/view/WindowManager.LayoutParams
// jni:  android/view/WindowManager$LayoutParams
// it also has some quirks:
// https://developer.android.com/training/articles/perf-jni.html#faq_FindClass
//

pub(crate) fn call_instance_method(name: &str) {
    log::debug!("Calling instance method from Rust: {}", name);
    let ctx = ndk_context::android_context();
    let vm = get_vm(&ctx);
    let mut env = get_env(&vm);
    let activity = unsafe { JObject::from_raw(ctx.context() as _) };
    if let Err(e) = env.call_method(activity, name, "()V", &[]) {
        log::error!("Error calling instance method {}: {}", name, e);
    }
}

pub(crate) fn get_vm(ctx: &ndk_context::AndroidContext) -> JavaVM {
    unsafe { JavaVM::from_raw(ctx.vm() as _) }.unwrap_or_else(|e| {
        log::error!("Error getting ctx.vm(): {:?}", e);
        panic!("No JavaVM found");
    })
}

pub(crate) fn get_env(vm: &JavaVM) -> AttachGuard {
    vm.attach_current_thread().unwrap_or_else(|e| {
        log::error!("Error attaching vm: {:?}", e);
        panic!("Failed to call attach_current_thread for JavaVM");
    })
}
