import AudioToolbox
import CoreHaptics
import Dispatch
import Foundation
import UIKit

@objc(HapticsPlugin)
public final class HapticsPlugin: NSObject {
  private lazy var lightImpactGenerator = UIImpactFeedbackGenerator(style: .light)
  private lazy var mediumImpactGenerator = UIImpactFeedbackGenerator(style: .medium)
  private lazy var heavyImpactGenerator = UIImpactFeedbackGenerator(style: .heavy)
  private lazy var softImpactGenerator = UIImpactFeedbackGenerator(style: .soft)
  private lazy var rigidImpactGenerator = UIImpactFeedbackGenerator(style: .rigid)
  private lazy var notificationGenerator = UINotificationFeedbackGenerator()
  private lazy var selectionGenerator = UISelectionFeedbackGenerator()
  private lazy var hapticEngine: CHHapticEngine? = makeHapticEngine()

  @objc public func vibrate(_ duration: Int64) {
    runOnMain {
      self.vibrateOnMain(duration)
    }
  }

  @objc public func impactFeedback(_ style: String) {
    runOnMain {
      let generator = self.impactGenerator(from: style)
      generator.prepare()
      generator.impactOccurred()
    }
  }

  @objc public func notificationFeedback(_ kind: String) {
    runOnMain {
      self.notificationGenerator.prepare()
      self.notificationGenerator.notificationOccurred(self.notificationType(from: kind))
    }
  }

  @objc public func selectionFeedback() {
    runOnMain {
      self.selectionGenerator.prepare()
      self.selectionGenerator.selectionChanged()
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
    guard let engine = hapticEngine else {
      AudioServicesPlayAlertSound(kSystemSoundID_Vibrate)
      return
    }

    do {
      try engine.start()

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
  }

  private func makeHapticEngine() -> CHHapticEngine? {
    guard CHHapticEngine.capabilitiesForHardware().supportsHaptics else {
      return nil
    }

    do {
      let engine = try CHHapticEngine()
      engine.resetHandler = {
        do {
          try engine.start()
        } catch {
          AudioServicesPlayAlertSound(kSystemSoundID_Vibrate)
        }
      }
      try engine.start()
      return engine
    } catch {
      return nil
    }
  }

  private func impactGenerator(from style: String) -> UIImpactFeedbackGenerator {
    switch style {
    case "light":
      return lightImpactGenerator
    case "heavy":
      return heavyImpactGenerator
    case "soft":
      return softImpactGenerator
    case "rigid":
      return rigidImpactGenerator
    default:
      return mediumImpactGenerator
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
