plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("rustPlugin")
    {{~#each android-app-plugins}}
    id("{{this}}"){{/each}}
}

android {
    compileSdk = 33
    defaultConfig {
        manifestPlaceholders["usesCleartextTraffic"] = "false"
        applicationId = "{{reverse-domain app.domain}}.{{snake-case app.name}}"
        minSdk = {{android.min-sdk-version}}
        targetSdk = 33
        versionCode = 1
        versionName = "1.0"
    }
    sourceSets.getByName("main") {
        {{#if android.vulkan-validation}}// Vulkan validation layers
        val ndkHome = System.getenv("NDK_HOME")
        jniLibs.srcDir("${ndkHome}/sources/third_party/vulkan/src/build-android/jniLibs")
        {{/if}}
    }
    buildTypes {
        getByName("debug") {
            manifestPlaceholders["usesCleartextTraffic"] = "true"
            isDebuggable = true
            isJniDebuggable = true
            isMinifyEnabled = false
            packagingOptions {
                {{~#each targets}}

                jniLibs.keepDebugSymbols.add("*/{{this.abi}}/*.so")
                {{/each}}
            }
        }
        getByName("release") {
            isMinifyEnabled = false
            proguardFiles(getDefaultProguardFile("proguard-android.txt"), "proguard-rules.pro")
        }
    }
    flavorDimensions.add("abi")
    productFlavors {
        create("universal") {
            val abiList = findProperty("abiList") as? String

            dimension = "abi"
            ndk {
                abiFilters += abiList?.split(",")?.map { it.trim() } ?: listOf(
                    {{~#each targets}}
                    "{{this.abi}}",{{/each}}
                )
            }
        }

        {{~#each targets}}

        create("{{this.arch}}") {
            dimension = "abi"
            ndk {
                abiFilters += listOf("{{this.abi}}")
            }
        }
        {{/each}}
    }

    assetPacks += mutableSetOf({{quote-and-join-colon-prefix asset-packs}})
}

rust {
    rootDirRel = "{{root-dir-rel}}"
    targets = listOf({{quote-and-join target-names}})
    arches = listOf({{quote-and-join arches}})
}

dependencies {
    {{~#each android-app-dependencies-platform}}
    implementation(platform("{{this}}")){{/each}}
    {{~#each android-app-dependencies}}
    implementation("{{this}}"){{/each}}
    implementation("androidx.webkit:webkit:1.4.0")
     implementation("androidx.appcompat:appcompat:1.5.0")
    implementation("com.google.android.material:material:1.6.1")
    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.1.3")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.4.0")
}

afterEvaluate {
    android.applicationVariants.all {
        tasks["mergeUniversalReleaseJniLibFolders"].dependsOn(tasks["rustBuildRelease"])
        tasks["mergeUniversalDebugJniLibFolders"].dependsOn(tasks["rustBuildDebug"])
        productFlavors.filter{ it.name != "universal" }.forEach { _ ->
            val archAndBuildType = name.capitalize()
            tasks["merge${archAndBuildType}JniLibFolders"].dependsOn(tasks["rustBuild${archAndBuildType}"])
        }
    }
}
