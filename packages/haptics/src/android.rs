use manganis::{
    android::with_activity,
    jni::{
        JNIEnv,
        objects::{GlobalRef, JClass, JObject},
    },
};
use std::sync::OnceLock;

use crate::{ImpactFeedbackStyle, NotificationFeedbackType};

const PLUGIN_CLASS: &str = "dev.dioxus.sdk.haptics.HapticsPlugin";
static PLUGIN: OnceLock<Result<GlobalRef, String>> = OnceLock::new();

// Bundle the Android Gradle module into the generated Dioxus app.
#[manganis::ffi("/android")]
unsafe extern "Kotlin" {}

fn find_plugin_class<'env>(
    env: &mut JNIEnv<'env>,
    activity: &JObject<'_>,
) -> Result<JClass<'env>, String> {
    let class_name = env
        .new_string(PLUGIN_CLASS)
        .map_err(|err| format!("failed to build plugin class name: {err:?}"))?;

    env.call_method(
        activity,
        "getAppClass",
        "(Ljava/lang/String;)Ljava/lang/Class;",
        &[(&class_name).into()],
    )
    .and_then(|class| class.l())
    .map(Into::into)
    .map_err(|err| format!("failed to load {PLUGIN_CLASS}: {err:?}"))
}

fn create_plugin() -> Result<GlobalRef, String> {
    let result = with_activity(|env, activity| {
        let class = match find_plugin_class(env, activity) {
            Ok(class) => class,
            Err(err) => return Some(Err(err)),
        };

        let context = match env
            .call_method(
                activity,
                "getApplicationContext",
                "()Landroid/content/Context;",
                &[],
            )
            .and_then(|context| context.l())
        {
            Ok(context) => context,
            Err(err) => {
                return Some(Err(format!(
                    "failed to access application context: {err:?}"
                )));
            }
        };

        let plugin =
            match env.new_object(&class, "(Landroid/content/Context;)V", &[(&context).into()]) {
                Ok(plugin) => plugin,
                Err(err) => {
                    return Some(Err(format!("failed to create HapticsPlugin: {err:?}")));
                }
            };

        match env.new_global_ref(&plugin) {
            Ok(plugin) => Some(Ok(plugin)),
            Err(err) => Some(Err(format!(
                "failed to create global plugin reference: {err:?}"
            ))),
        }
    });

    match result {
        Some(result) => result,
        None => Err("failed to access Android activity".to_string()),
    }
}

fn plugin() -> Result<&'static GlobalRef, String> {
    PLUGIN
        .get_or_init(create_plugin)
        .as_ref()
        .map_err(Clone::clone)
}

fn with_plugin<R>(
    f: impl FnOnce(&mut JNIEnv<'_>, &GlobalRef) -> Result<R, String>,
) -> Result<R, String> {
    let plugin = plugin()?;
    let result = with_activity(|env, _activity| Some(f(env, plugin)));

    match result {
        Some(result) => result,
        None => Err("failed to access Android activity".to_string()),
    }
}

pub fn vibrate(duration: u32) -> Result<(), String> {
    with_plugin(|env, plugin| {
        env.call_method(
            plugin.as_obj(),
            "vibrate",
            "(J)V",
            &[i64::from(duration).into()],
        )
        .map_err(|err| format!("failed to vibrate: {err:?}"))?;
        Ok(())
    })
}

pub fn impact_feedback(style: ImpactFeedbackStyle) -> Result<(), String> {
    with_plugin(|env, plugin| {
        let style = env
            .new_string(style.to_string())
            .map_err(|err| format!("failed to build impact style string: {err:?}"))?;

        env.call_method(
            plugin.as_obj(),
            "impactFeedback",
            "(Ljava/lang/String;)V",
            &[(&style).into()],
        )
        .map_err(|err| format!("failed to run impact feedback: {err:?}"))?;

        Ok(())
    })
}

pub fn notification_feedback(kind: NotificationFeedbackType) -> Result<(), String> {
    with_plugin(|env, plugin| {
        let kind = env
            .new_string(kind.to_string())
            .map_err(|err| format!("failed to build notification type string: {err:?}"))?;

        env.call_method(
            plugin.as_obj(),
            "notificationFeedback",
            "(Ljava/lang/String;)V",
            &[(&kind).into()],
        )
        .map_err(|err| format!("failed to run notification feedback: {err:?}"))?;

        Ok(())
    })
}

pub fn selection_feedback() -> Result<(), String> {
    with_plugin(|env, plugin| {
        env.call_method(plugin.as_obj(), "selectionFeedback", "()V", &[])
            .map_err(|err| format!("failed to run selection feedback: {err:?}"))?;
        Ok(())
    })
}
