# flutter_local_notifications uses Gson reflection to persist scheduled
# notifications. Keep the generic signatures and plugin types required by
# TypeToken / RuntimeTypeAdapterFactory.
-keepattributes Signature
-keepattributes *Annotation*

-keep class com.google.gson.** { *; }
-keep class com.google.gson.stream.** { *; }
-keep class com.google.gson.reflect.TypeToken { *; }
-keepclassmembers class * {
  @com.google.gson.annotations.SerializedName <fields>;
}
-keepclassmembers class * {
  @com.google.gson.annotations.Expose <fields>;
}
-keepclassmembers enum * {
  public static **[] values();
  public ** valueOf(java.lang.String);
}

-keep class com.dexterous.flutterlocalnotifications.** { *; }
-keepclassmembers class com.dexterous.flutterlocalnotifications.** { *; }
