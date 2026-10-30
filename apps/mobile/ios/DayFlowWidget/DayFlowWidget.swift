import AppIntents
import SwiftUI
import WidgetKit

private let appGroup = "group.com.dayflow.dayflowMobile"
private let widgetKind = "DayFlowWidget"

private struct WidgetTask: Decodable, Identifiable {
  let id: String
  let title: String
  let time: String?
}

private struct TaskSnapshot: Decodable {
  let today: [WidgetTask]
  let legacyCount: Int
  let legacy: [WidgetTask]

  static func load() -> TaskSnapshot {
    let data = UserDefaults(suiteName: appGroup)?.string(forKey: "dayFlowTasks") ?? ""
    guard let decoded = data.data(using: .utf8),
      let snapshot = try? JSONDecoder().decode(TaskSnapshot.self, from: decoded)
    else {
      return TaskSnapshot(today: [], legacyCount: 0, legacy: [])
    }
    return snapshot
  }
}

private struct TaskEntry: TimelineEntry {
  let date: Date
  let snapshot: TaskSnapshot
}

private struct TaskProvider: TimelineProvider {
  func placeholder(in context: Context) -> TaskEntry {
    TaskEntry(date: .now, snapshot: TaskSnapshot(today: [], legacyCount: 0, legacy: []))
  }

  func getSnapshot(in context: Context, completion: @escaping (TaskEntry) -> Void) {
    completion(TaskEntry(date: .now, snapshot: .load()))
  }

  func getTimeline(in context: Context, completion: @escaping (Timeline<TaskEntry>) -> Void) {
    let entry = TaskEntry(date: .now, snapshot: .load())
    completion(Timeline(entries: [entry], policy: .after(.now.addingTimeInterval(30 * 60))))
  }
}

private struct TaskRow: View {
  let task: WidgetTask
  let palette: WidgetPalette

  var body: some View {
    HStack(spacing: 10) {
      if #available(iOSApplicationExtension 17.0, *) {
        Button(intent: CompleteWidgetTaskIntent(taskID: task.id)) {
          Image(systemName: "circle")
            .font(.system(size: 19, weight: .regular))
            .foregroundStyle(palette.accent)
        }
        .buttonStyle(.plain)
        .accessibilityLabel("完成\(task.title)")
      } else {
        Link(destination: URL(string: "dayflow://widget")!) {
          Image(systemName: "circle")
            .font(.system(size: 19, weight: .regular))
            .foregroundStyle(palette.accent)
        }
        .accessibilityLabel("打开 DayFlow")
      }

      Text(task.title)
        .lineLimit(1)
        .frame(maxWidth: .infinity, alignment: .leading)
        .font(.system(size: 13, weight: .medium))
        .foregroundStyle(palette.primaryText)
      Text(task.time ?? "待安排")
        .font(.system(size: 10, weight: .medium))
        .foregroundStyle(palette.secondaryText)
        .padding(.horizontal, 7)
        .padding(.vertical, 4)
        .background(palette.badge, in: Capsule())
    }
    .padding(.vertical, 3)
  }
}

private struct WidgetPalette {
  let isDark: Bool

  var accent: Color { Color(red: 0.42, green: 0.49, blue: 0.94) }
  var primaryText: Color {
    isDark ? Color(red: 0.94, green: 0.95, blue: 0.99) : Color(red: 0.13, green: 0.16, blue: 0.27)
  }
  var secondaryText: Color {
    isDark ? Color(red: 0.70, green: 0.72, blue: 0.80) : Color(red: 0.46, green: 0.49, blue: 0.59)
  }
  var badge: Color {
    isDark ? Color(red: 0.19, green: 0.21, blue: 0.32) : Color(red: 0.93, green: 0.94, blue: 0.98)
  }
  var surface: Color {
    isDark ? Color(red: 0.10, green: 0.11, blue: 0.16) : Color(red: 0.98, green: 0.98, blue: 1.0)
  }
}

private struct TaskWidgetView: View {
  let entry: TaskEntry
  @Environment(\.widgetFamily) private var family
  @Environment(\.colorScheme) private var colorScheme

  private var palette: WidgetPalette {
    WidgetPalette(isDark: colorScheme == .dark)
  }

  var body: some View {
    VStack(alignment: .leading, spacing: 0) {
      HStack(spacing: 9) {
        Text("D")
          .font(.system(size: 15, weight: .bold))
          .foregroundStyle(.white)
          .frame(width: 31, height: 31)
          .background(palette.accent, in: RoundedRectangle(cornerRadius: 10))
        VStack(alignment: .leading, spacing: 2) {
          Link(destination: URL(string: "dayflow://widget")!) {
            Text("DayFlow")
              .font(.system(size: 15, weight: .bold))
              .foregroundStyle(palette.primaryText)
          }
          Text("今天，轻松安排")
            .font(.system(size: 10))
            .foregroundStyle(palette.secondaryText)
        }
        Spacer(minLength: 4)
        Text("今日")
          .font(.system(size: 10, weight: .semibold))
          .foregroundStyle(palette.accent)
          .padding(.horizontal, 9)
          .padding(.vertical, 5)
          .background(palette.badge, in: Capsule())
      }

      Rectangle()
        .fill(palette.badge)
        .frame(height: 1)
        .padding(.top, 11)
        .padding(.bottom, 6)

      if entry.snapshot.today.isEmpty {
        HStack(spacing: 7) {
          Image(systemName: "checkmark.circle")
            .foregroundStyle(palette.accent)
          Text("今天还没有任务")
            .font(.system(size: 12))
            .foregroundStyle(palette.secondaryText)
        }
        .padding(.vertical, 8)
      } else {
        ForEach(entry.snapshot.today.prefix(family == .systemSmall ? 2 : 4)) {
          TaskRow(task: $0, palette: palette)
        }
      }

      Spacer(minLength: 0)
      HStack(spacing: 6) {
        Image(systemName: "clock.arrow.circlepath")
          .font(.system(size: 11, weight: .medium))
        Text("之前未完成")
          .font(.system(size: 10, weight: .semibold))
        Text("\(entry.snapshot.legacyCount)")
          .font(.system(size: 10, weight: .bold))
          .foregroundStyle(palette.accent)
          .padding(.horizontal, 6)
          .padding(.vertical, 3)
          .background(palette.badge, in: Capsule())
      }
      .foregroundStyle(palette.secondaryText)
      if family != .systemSmall && !entry.snapshot.legacy.isEmpty {
        Text(entry.snapshot.legacy.prefix(2).map(\.title).joined(separator: " · "))
          .font(.system(size: 10))
          .foregroundStyle(palette.secondaryText)
          .lineLimit(1)
          .padding(.top, 4)
      }
    }
    .padding(14)
    .background {
      if #available(iOSApplicationExtension 17.0, *) {
        Color.clear
      } else {
        palette.surface
      }
    }
    .applyWidgetContainerBackground(palette.surface)
  }
}

private extension View {
  @ViewBuilder
  func applyWidgetContainerBackground(_ background: Color) -> some View {
    if #available(iOSApplicationExtension 17.0, *) {
      containerBackground(background, for: .widget)
    } else {
      self.background(background)
    }
  }
}

@available(iOSApplicationExtension 16.0, *)
struct CompleteWidgetTaskIntent: AppIntent {
  static var title: LocalizedStringResource = "完成 DayFlow 任务"
  static var openAppWhenRun = false

  @Parameter(title: "Task ID")
  var taskID: String

  init() {}

  init(taskID: String) {
    self.taskID = taskID
  }

  func perform() async throws -> some IntentResult {
    guard let defaults = UserDefaults(suiteName: appGroup) else {
      throw WidgetActionError.appGroupUnavailable
    }

    let snapshot = TaskSnapshot.load()
    let remaining = snapshot.today.filter { $0.id != taskID }
    let encoded = try JSONSerialization.data(withJSONObject: [
      "today": remaining.map { ["id": $0.id, "title": $0.title, "time": $0.time as Any? ?? NSNull()] },
      "legacyCount": snapshot.legacyCount,
      "legacy": snapshot.legacy.map { ["id": $0.id, "title": $0.title, "time": $0.time as Any? ?? NSNull()] },
    ])
    let pending = defaults.string(forKey: "pendingCompletionIds") ?? "[]"
    var pendingIds = (try? JSONDecoder().decode([String].self, from: Data(pending.utf8))) ?? []
    if !pendingIds.contains(taskID) { pendingIds.append(taskID) }
    defaults.set(String(data: encoded, encoding: .utf8), forKey: "dayFlowTasks")
    defaults.set(String(data: try JSONEncoder().encode(pendingIds), encoding: .utf8), forKey: "pendingCompletionIds")
    WidgetCenter.shared.reloadTimelines(ofKind: widgetKind)
    return .result()
  }
}

private enum WidgetActionError: Error {
  case appGroupUnavailable
}

private struct DayFlowWidget: Widget {
  var body: some WidgetConfiguration {
    StaticConfiguration(kind: widgetKind, provider: TaskProvider()) { entry in
      TaskWidgetView(entry: entry)
    }
    .configurationDisplayName("DayFlow 任务")
    .description("查看今日任务和之前未完成的任务，可快速完成今日任务。")
    .supportedFamilies([.systemSmall, .systemMedium, .systemLarge])
  }
}

@main
private struct DayFlowWidgetBundle: WidgetBundle {
  var body: some Widget {
    DayFlowWidget()
  }
}
