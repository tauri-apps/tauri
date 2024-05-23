plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("rust")
    {{~#each android-app-plugins}}
    id("{{this}}"){{/each}}
}

android {
    compileSdk = 33
    namespace = "{{reverse-domain app.domain}}"
    defaultConfig {
        manifestPlaceholders["usesCleartextTraffic"] = "false"
        applicationId = "{{reverse-domain app.domain}}"
        minSdk = {{android.min-sdk-version}}
        targetSdk = 33
        versionCode = 1
        versionName = "1.0"
    }
    buildTypes {
        getByName("debug") {
            manifestPlaceholders["usesCleartextTraffic"] = "true"
            isDebuggable = true
            isJniDebuggable = true
            isMinifyEnabled = false
            packaging {
                {{~#each abi-list}}
                jniLibs.keepDebugSymbols.add("*/{{this}}/*.so")
                {{/each}}
            }
        }
        getByName("release") {
            isMinifyEnabled = true
            proguardFiles(
                *fileTree(".") { include("**/*.pro") }
                    .plus(getDefaultProguardFile("proguard-android-optimize.txt"))
                    .toList().toTypedArray()
            )
        }
    }
    kotlinOptions {
        jvmTarget = "1.8"
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
    implementation("androidx.webkit:webkit:1.6.1")
    implementation("androidx.appcompat:appcompat:1.6.1")
    implementation("com.google.android.material:material:1.8.0")
    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.1.4")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.5.0")
}

apply(from = "tauri.build.gradle.kts")