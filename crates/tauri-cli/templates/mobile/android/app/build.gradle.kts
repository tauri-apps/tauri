import java.util.Properties
import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("rust")
    {{~#each android-app-plugins}}
    id("{{this}}"){{/each}}
}

val tauriProperties = Properties().apply {
    val propFile = file("tauri.properties")
    if (propFile.exists()) {
        propFile.inputStream().use { load(it) }
    }
}

android {
    compileSdk = 37
    namespace = "{{app.identifier}}"
    defaultConfig {
        manifestPlaceholders["usesCleartextTraffic"] = "false"
        applicationId = "{{app.identifier}}"
        minSdk = {{android.min-sdk-version}}
        targetSdk = 37
        versionCode = tauriProperties.getProperty("tauri.android.versionCode", "1").toInt()
        versionName = tauriProperties.getProperty("tauri.android.versionName", "1.0")
    }
    buildTypes {
        getByName("debug") {
            {{#if android-debug-application-id-suffix}}
            applicationIdSuffix = "{{android-debug-application-id-suffix}}"
            {{/if}}
            manifestPlaceholders["usesCleartextTraffic"] = "true"
            isDebuggable = true
            isJniDebuggable = true
            isMinifyEnabled = false
        }
        getByName("release") {
            optimization {
               enable = true
            }
            proguardFiles(
                *fileTree(".") {
                  include("**/*.pro")
                  exclude("build/**")
                }.files.toTypedArray()
            )
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }
    buildFeatures {
        buildConfig = true
    }
}

// `packaging` has no equivalent in the `buildTypes { getByName("debug") { ... } }` DSL, so a
// `packaging { ... }` block placed there silently resolves against the outer `android {}`
// extension instead and applies to every build type, including release. Scope it to the debug
// variant explicitly via the variant API so release libs are still stripped and
// `ndk.debugSymbolLevel` can extract debug metadata.
androidComponents {
    onVariants(selector().withBuildType("debug")) { variant ->
        {{#each abi-list}}
        variant.packaging.jniLibs.keepDebugSymbols.add("*/{{this}}/*.so")
        {{/each}}
    }
}

kotlin {
    compilerOptions {
        jvmTarget = JvmTarget.JVM_1_8
    }
}

rust {
    rootDirRel = "{{root-dir-rel}}"
}

dependencies {
    {{~#each android-app-dependencies-platform}}
    implementation(platform("{{this}}")){{/each}}
    {{~#each android-app-dependencies}}
    implementation("{{this}}"){{/each}}
    implementation("androidx.webkit:webkit:1.14.0")
    implementation("androidx.appcompat:appcompat:1.7.1")
    implementation("androidx.activity:activity-ktx:1.10.1")
    implementation("com.google.android.material:material:1.12.0")
    implementation("androidx.lifecycle:lifecycle-process:2.10.0")
    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.1.4")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.5.0")
}

apply(from = file("tauri.build.gradle.kts"))
