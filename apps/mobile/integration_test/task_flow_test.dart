import 'package:dayflow_mobile/src/application/task_service.dart';
import 'package:dayflow_mobile/src/domain/task.dart';
import 'package:dayflow_mobile/src/home_page.dart';
import 'package:dayflow_mobile/src/platform/task_widget_service.dart';
import 'package:dayflow_mobile/src/platform/widget_background_callback.dart';
import 'package:dayflow_mobile/src/repo/task_repository.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:home_widget/home_widget.dart';
import 'package:integration_test/integration_test.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('widget completion keeps the foreground database usable',
      (tester) async {
    final repository = TaskRepository();
    final service = TaskService(repository);
    final title = 'platform_${DateTime.now().microsecondsSinceEpoch}';

    await tester.pumpWidget(MaterialApp(home: HomePage(service: service)));
    await tester.pumpAndSettle();
    await tester.tap(find.byTooltip('添加任务'));
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextField), title);
    await tester.tap(find.text('添加').last);
    await tester.pumpAndSettle();

    expect(find.text(title), findsOneWidget);

    final created = (await repository.getActiveTasks())
        .singleWhere((task) => task.title == title);
    await TaskWidgetService().publish(await repository.getActiveTasks());
    await handleDayFlowWidgetAction(
      Uri.parse('dayflow://complete?id=${created.id}'),
    );

    final savedTask = (await repository.getActiveTasks()).singleWhere(
      (task) => task.title == title,
    );
    expect(savedTask.status, TaskStatus.completed);
    expect(savedTask.scheduledTime, isNull);
    final widgetData = await HomeWidget.getWidgetData<String>('dayFlowTasks');
    expect(widgetData, isNot(contains(title)));
    await service.softDeleteTask(savedTask);
    await repository.close();
  });
}
