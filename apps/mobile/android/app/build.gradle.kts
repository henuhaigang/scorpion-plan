plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    // The Flutter Gradle Plugin must be applied after the Android and Kotlin Gradle plugins.
    id("dev.flutter.flutter-gradle-plugin")
}

val localKeystoreFile = rootProject.file("keystore/splan-release.jks")
val envKeystorePath = providers.environmentVariable("ANDROID_KEYSTORE_PATH").orNull
    ?: localKeystoreFile.takeIf { it.exists() }?.absolutePath
val envStorePassword = providers.environmentVariable("ANDROID_STORE_PASSWORD").orNull
val envKeyAlias = providers.environmentVariable("ANDROID_KEY_ALIAS").orNull
val envKeyPassword = providers.environmentVariable("ANDROID_KEY_PASSWORD").orNull
val hasEnvSigning = listOf(envKeystorePath, envStorePassword, envKeyAlias, envKeyPassword)
    .all { !it.isNullOrBlank() }

android {
    namespace = "com.splan.splan_mobile"
    compileSdk = flutter.compileSdkVersion
    ndkVersion = flutter.ndkVersion

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
        isCoreLibraryDesugaringEnabled = true
    }

    defaultConfig {
        // TODO: Specify your own unique Application ID (https://developer.android.com/studio/build/application-id.html).
        applicationId = "com.splan.splan_mobile"
        // You can update the following values to match your application needs.
        // For more information, see: https://flutter.dev/to/review-gradle-config.
        minSdk = flutter.minSdkVersion
        targetSdk = flutter.targetSdkVersion
        // Uses the version code from pubspec.yaml. When using split APKs, 1000 * ABI_VERSION
        // is added automatically by Flutter. (https://developer.android.com/studio/build/configure-apk-splits#configure-APK-versions)
        // You can force using the value of versionCode by specifying the `-P force-version-code-ignoring-abi=true`
        // flag during build.
        versionCode = flutter.versionCode
        versionName = flutter.versionName
    }

    signingConfigs {
        if (hasEnvSigning) {
            create("release") {
                storeFile = rootProject.file(requireNotNull(envKeystorePath))
                storePassword = requireNotNull(envStorePassword)
                keyAlias = requireNotNull(envKeyAlias)
                keyPassword = requireNotNull(envKeyPassword)
            }
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = true
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro",
            )
            val releaseConfig = signingConfigs.findByName("release")
            if (releaseConfig != null) {
                signingConfig = releaseConfig
            }
        }
    }
}

dependencies {
    coreLibraryDesugaring("com.android.tools:desugar_jdk_libs:2.1.4")
}

kotlin {
    compilerOptions {
        jvmTarget = org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17
    }
}

flutter {
    source = "../.."
}
