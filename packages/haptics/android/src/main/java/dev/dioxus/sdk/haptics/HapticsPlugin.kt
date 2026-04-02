package dev.dioxus.sdk.haptics

import android.content.Context
import android.os.Build
import android.os.VibrationEffect
import android.os.Vibrator
import android.os.VibratorManager
import dev.dioxus.sdk.haptics.patterns.ImpactPatternHeavy
import dev.dioxus.sdk.haptics.patterns.ImpactPatternLight
import dev.dioxus.sdk.haptics.patterns.ImpactPatternMedium
import dev.dioxus.sdk.haptics.patterns.ImpactPatternRigid
import dev.dioxus.sdk.haptics.patterns.ImpactPatternSoft
import dev.dioxus.sdk.haptics.patterns.NotificationPatternError
import dev.dioxus.sdk.haptics.patterns.NotificationPatternSuccess
import dev.dioxus.sdk.haptics.patterns.NotificationPatternWarning
import dev.dioxus.sdk.haptics.patterns.Pattern
import dev.dioxus.sdk.haptics.patterns.SelectionPattern

class HapticsPlugin(context: Context) {
    private val appContext = context.applicationContext

    private val vibrator: Vibrator = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
        val vibratorManager =
            appContext.getSystemService(Context.VIBRATOR_MANAGER_SERVICE) as VibratorManager
        vibratorManager.defaultVibrator
    } else {
        @Suppress("DEPRECATION")
        appContext.getSystemService(Context.VIBRATOR_SERVICE) as Vibrator
    }

    fun vibrate(duration: Long) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            vibrator.vibrate(VibrationEffect.createOneShot(duration, VibrationEffect.DEFAULT_AMPLITUDE))
        } else {
            @Suppress("DEPRECATION")
            vibrator.vibrate(duration)
        }
    }

    fun impactFeedback(style: String) {
        vibratePattern(
            when (style.lowercase()) {
                "light" -> ImpactPatternLight
                "heavy" -> ImpactPatternHeavy
                "soft" -> ImpactPatternSoft
                "rigid" -> ImpactPatternRigid
                else -> ImpactPatternMedium
            }
        )
    }

    fun notificationFeedback(kind: String) {
        vibratePattern(
            when (kind.lowercase()) {
                "warning" -> NotificationPatternWarning
                "error" -> NotificationPatternError
                else -> NotificationPatternSuccess
            }
        )
    }

    fun selectionFeedback() {
        vibratePattern(SelectionPattern)
    }

    private fun vibratePattern(pattern: Pattern) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            vibrator.vibrate(VibrationEffect.createWaveform(pattern.timings, pattern.amplitudes, -1))
        } else {
            @Suppress("DEPRECATION")
            vibrator.vibrate(pattern.oldSDKPattern, -1)
        }
    }
}
