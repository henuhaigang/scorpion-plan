package com.splan.splan_mobile

import android.appwidget.AppWidgetManager
import android.content.Context
import android.content.SharedPreferences
import android.net.Uri
import android.os.Bundle
import android.widget.RemoteViews
import es.antonborri.home_widget.HomeWidgetBackgroundIntent
import es.antonborri.home_widget.HomeWidgetLaunchIntent
import es.antonborri.home_widget.HomeWidgetPlugin
import es.antonborri.home_widget.HomeWidgetProvider
import org.json.JSONObject

class SplanWidgetProvider : HomeWidgetProvider() {
    override fun onUpdate(
        context: Context,
        appWidgetManager: AppWidgetManager,
        appWidgetIds: IntArray,
        widgetData: SharedPreferences,
    ) {
        updateWidgets(context, appWidgetManager, appWidgetIds, widgetData)
    }

    override fun onAppWidgetOptionsChanged(
        context: Context,
        appWidgetManager: AppWidgetManager,
        appWidgetId: Int,
        newOptions: Bundle,
    ) {
        super.onAppWidgetOptionsChanged(context, appWidgetManager, appWidgetId, newOptions)
        updateWidgets(
            context,
            appWidgetManager,
            intArrayOf(appWidgetId),
            HomeWidgetPlugin.getData(context),
        )
    }

    private fun updateWidgets(
        context: Context,
        appWidgetManager: AppWidgetManager,
        appWidgetIds: IntArray,
        widgetData: SharedPreferences,
    ) {
        val snapshot = runCatching {
            JSONObject(widgetData.getString("splanTasks", "{}") ?: "{}")
        }.getOrElse { JSONObject() }
        val today = snapshot.optJSONArray("today")
        val legacyCount = snapshot.optInt("legacyCount")
        val legacy = snapshot.optJSONArray("legacy")

        appWidgetIds.forEach { widgetId ->
            val views = RemoteViews(context.packageName, R.layout.splan_widget)
            val heightDp = appWidgetManager
                .getAppWidgetOptions(widgetId)
                .getInt(AppWidgetManager.OPTION_APPWIDGET_MIN_HEIGHT, 260)
            val visibleTaskCount = when {
                heightDp < 160 -> 1
                heightDp < 205 -> 2
                heightDp < 250 -> 3
                else -> 4
            }
            val showLegacy = heightDp >= 220

            views.setViewVisibility(
                R.id.widget_subtitle,
                if (heightDp < 160) android.view.View.GONE else android.view.View.VISIBLE,
            )
            views.setViewVisibility(
                R.id.widget_legacy_container,
                if (showLegacy) android.view.View.VISIBLE else android.view.View.GONE,
            )
            views.setTextViewText(
                R.id.widget_legacy_count,
                "之前未完成 · $legacyCount",
            )
            views.setTextViewText(R.id.widget_today_badge, "今日")
            views.setTextViewText(R.id.widget_legacy_titles, legacyTitles(legacy))
            views.setOnClickPendingIntent(
                R.id.widget_root,
                HomeWidgetLaunchIntent.getActivity(
                    context,
                    MainActivity::class.java,
                    Uri.parse("splan://widget"),
                ),
            )
            val rowIds = listOf(
                Triple(R.id.widget_task_1, R.id.widget_title_1, R.id.widget_time_1),
                Triple(R.id.widget_task_2, R.id.widget_title_2, R.id.widget_time_2),
                Triple(R.id.widget_task_3, R.id.widget_title_3, R.id.widget_time_3),
                Triple(R.id.widget_task_4, R.id.widget_title_4, R.id.widget_time_4),
            )
            rowIds.forEachIndexed { index, (rowId, titleId, timeId) ->
                val task = today?.optJSONObject(index)
                if (task == null || index >= visibleTaskCount) {
                    views.setViewVisibility(rowId, android.view.View.GONE)
                } else {
                    views.setViewVisibility(rowId, android.view.View.VISIBLE)
                    views.setTextViewText(titleId, task.optString("title"))
                    val time = if (task.isNull("time")) {
                        null
                    } else {
                        task.optString("time").takeIf { it.isNotBlank() }
                    }
                    views.setTextViewText(timeId, time ?: "待安排")
                    val taskId = task.optString("id")
                    views.setOnClickPendingIntent(
                        rowId,
                        HomeWidgetBackgroundIntent.getBroadcast(
                            context,
                            Uri.parse("splan://complete?id=${Uri.encode(taskId)}"),
                        ),
                    )
                    views.setContentDescription(
                        rowId,
                        "完成任务：${task.optString("title")}",
                    )
                }
            }
            appWidgetManager.updateAppWidget(widgetId, views)
        }
    }

    private fun legacyTitles(legacy: org.json.JSONArray?): String {
        if (legacy == null || legacy.length() == 0) return "没有遗留任务"
        return (0 until legacy.length())
            .mapNotNull { legacy.optJSONObject(it)?.optString("title") }
            .filter { it.isNotBlank() }
            .joinToString(" · ")
    }
}
