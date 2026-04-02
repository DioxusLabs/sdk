import AudioToolbox
import CoreHaptics
import Dispatch
import Foundation
import UIKit

@objc(HapticsPlugin)
public final class HapticsPlugin: NSObject {
  @objc public func vibrate(_ duration: Int64) {
    runOnMain {
      self.vibrateOnMain(duration)
    }
  }

  @objc public func impactFeedback(_ style: String) {
    runOnMain {
      let generator = UIImpactFeedbackGenerator(style: self.impactStyle(from: style))
      generator.prepare()
      generator.impactOccurred()
    }
  }

  @objc public func notificationFeedback(_ kind: String) {
    runOnMain {
      let generator = UINotificationFeedbackGenerator()
      generator.prepare()
      generator.notificationOccurred(self.notificationType(from: kind))
    }
  }

  @objc public func selectionFeedback() {
    runOnMain {
      let generator = UISelectionFeedbackGenerator()
      generator.prepare()
      generator.selectionChanged()
    }
  }

  private func runOnMain(_ action: @escaping () -> Void) {
    if Thread.isMainThread {
      action()
    } else {
      DispatchQueue.main.sync(execute: action)
    }
  }

  private func vibrateOnMain(_ duration: Int64) {
    if #available(iOS 13.0, *), CHHapticEngine.capabilitiesForHardware().supportsHaptics {
      do {
        let engine = try CHHapticEngine()
        try engine.start()

        engine.resetHandler = {
          do {
            try engine.start()
          } catch {
            AudioServicesPlayAlertSound(kSystemSoundID_Vibrate)
          }
        }

        let intensity = CHHapticEventParameter(
          parameterID: .hapticIntensity,
          value: 1.0
        )
        let sharpness = CHHapticEventParameter(
          parameterID: .hapticSharpness,
          value: 1.0
        )
        let event = CHHapticEvent(
          eventType: .hapticContinuous,
          parameters: [intensity, sharpness],
          relativeTime: 0.0,
          duration: Double(duration) / 1000.0
        )
        let pattern = try CHHapticPattern(events: [event], parameters: [])
        let player = try engine.makePlayer(with: pattern)

        try player.start(atTime: 0)
      } catch {
        AudioServicesPlayAlertSound(kSystemSoundID_Vibrate)
      }
    } else {
      AudioServicesPlayAlertSound(kSystemSoundID_Vibrate)
    }
  }

  private func impactStyle(from style: String) -> UIImpactFeedbackGenerator.FeedbackStyle {
    switch style {
    case "light":
      return .light
    case "heavy":
      return .heavy
    case "soft":
      return .soft
    case "rigid":
      return .rigid
    default:
      return .medium
    }
  }

  private func notificationType(from kind: String) -> UINotificationFeedbackGenerator.FeedbackType {
    switch kind {
    case "warning":
      return .warning
    case "error":
      return .error
    default:
      return .success
    }
  }
}
