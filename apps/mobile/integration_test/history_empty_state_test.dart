import 'package:splan_mobile/src/application/task_service.dart';
import 'package:splan_mobile/src/domain/task.dart';
import 'package:splan_mobile/src/home_page.dart';
import 'package:splan_mobile/src/repo/task_repository.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

void main() {
  final binding = IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('empty history fills the page without a floating card',
      (tester) async {
    await tester.pumpWidget(
      MaterialApp(home: HomePage(service: TaskService(_EmptyTaskStore()))),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.byTooltip('历史'));
    await tester.pumpAndSettle();

    final emptyPage = find.byKey(const ValueKey('history-empty-page'));
    expect(find.text('暂无历史记录'), findsOneWidget);
    expect(tester.getSize(emptyPage).width, greaterThan(300));
    expect(tester.getSize(emptyPage).height, greaterThan(600));
    expect(
      find.descendant(of: emptyPage, matching: find.byType(Card)),
      findsNothing,
    );

    await binding.convertFlutterSurfaceToImage();
    await tester.pump();
    await binding.takeScreenshot('history-empty-page');
  });
}

class _EmptyTaskStore implements TaskStore {
  @override
  Future<List<Task>> getActiveTasks() async => [];

  @override
  Future<Task?> getTask(String id) async => null;

  @override
  Future<void> insertTask(Task task) async {
    throw UnsupportedError('The empty-state test does not create tasks.');
  }

  @override
  Future<void> updateTask(Task task, {required int expectedVersion}) async {
    throw UnsupportedError('The empty-state test does not update tasks.');
  }

  @override
  Future<void> close() async {}
}
