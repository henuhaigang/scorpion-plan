import 'package:flutter/material.dart';
import 'home_page.dart';

class SplanApp extends StatelessWidget {
  const SplanApp({super.key});

  ThemeData _theme(Brightness brightness) {
    const seedColor = Color(0xFF5B63C8);
    final colorScheme = ColorScheme.fromSeed(
      seedColor: seedColor,
      brightness: brightness,
    );
    final isLight = brightness == Brightness.light;
    final pageColor = isLight ? const Color(0xFFF6F7FB) : colorScheme.surface;

    return ThemeData(
      colorScheme: colorScheme,
      useMaterial3: true,
      scaffoldBackgroundColor: pageColor,
      appBarTheme: AppBarTheme(
        backgroundColor: pageColor,
        foregroundColor: colorScheme.onSurface,
        elevation: 0,
        scrolledUnderElevation: 0,
        surfaceTintColor: Colors.transparent,
        toolbarHeight: 68,
        titleTextStyle: TextStyle(
          color: colorScheme.onSurface,
          fontSize: 21,
          fontWeight: FontWeight.w700,
          letterSpacing: -0.5,
        ),
      ),
      cardTheme: CardThemeData(
        color: isLight ? Colors.white : colorScheme.surfaceContainerLow,
        elevation: 0,
        margin: EdgeInsets.zero,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(20),
          side: BorderSide(
            color: colorScheme.outlineVariant.withValues(
              alpha: isLight ? 0.48 : 0.28,
            ),
          ),
        ),
      ),
      dialogTheme: DialogThemeData(
        backgroundColor:
            isLight ? Colors.white : colorScheme.surfaceContainerLow,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(24),
        ),
      ),
      inputDecorationTheme: InputDecorationTheme(
        filled: true,
        fillColor:
            isLight ? const Color(0xFFF6F7FB) : colorScheme.surfaceContainer,
        border: OutlineInputBorder(
          borderRadius: BorderRadius.circular(16),
          borderSide: BorderSide.none,
        ),
        enabledBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(16),
          borderSide: BorderSide(
            color: colorScheme.outlineVariant.withValues(alpha: 0.6),
          ),
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(16),
          borderSide: BorderSide(color: colorScheme.primary, width: 1.5),
        ),
      ),
      filledButtonTheme: FilledButtonThemeData(
        style: FilledButton.styleFrom(
          minimumSize: const Size(48, 48),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(16),
          ),
          textStyle: const TextStyle(fontWeight: FontWeight.w600),
        ),
      ),
      snackBarTheme: SnackBarThemeData(
        behavior: SnackBarBehavior.floating,
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Splan',
      debugShowCheckedModeBanner: false,
      theme: _theme(Brightness.light),
      darkTheme: _theme(Brightness.dark),
      themeMode: ThemeMode.system,
      home: const HomePage(),
    );
  }
}
