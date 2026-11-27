import com.android.build.api.dsl.ApplicationExtension

val cefDir = file(providers.environmentVariable("CEF_PATH").get())
val chromiumDir = cefDir.parentFile.parentFile

extensions.configure<ApplicationExtension> {
    defaultConfig.minSdk = maxOf(defaultConfig.minSdk ?: 0, 29)
    androidResources.noCompress += listOf("pak", "dat", "bin")
    bundle.language.enableSplit = true
}

dependencies {
    add("implementation", "androidx.games:games-activity:4.4.0")
    add("implementation", files(cefDir.resolve("cef.aar")))
    add("implementation", "androidx.biometric:biometric:1.1.0")
    add("implementation", "com.google.android.material:material:1.14.0")
    add("implementation", "com.google.android.gms:play-services-iid:17.0.0")
    add("implementation", "com.google.guava:guava:33.7.1-jre")
    // Chromium's Java code calls APIs from its pinned Core snapshot.
    add("implementation", files(chromiumDir.resolve("third_party/androidx/cipd/libs/androidx_core_core/core.aar")))
    add("implementation", files(chromiumDir.resolve("third_party/androidx/cipd/libs/androidx_core_core_viewtree/core-viewtree.aar")))
}

configurations.configureEach {
    exclude(group = "androidx.core", module = "core")
    // KTX is included in Chromium's version of Core.
    exclude(group = "androidx.core", module = "core-ktx")
    exclude(group = "androidx.core", module = "core-viewtree")
}
