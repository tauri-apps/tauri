package {{reverse-domain app.domain}}

import com.android.build.gradle.*
import java.io.File
import org.gradle.api.DefaultTask
import org.gradle.api.GradleException
import org.gradle.api.Plugin
import org.gradle.api.Project

const val TASK_GROUP = "rust"

open class Config {
    var rootDirRel: String? = null
    var targets: List<String>? = null
    var arches: List<String>? = null
}

open class RustPlugin : Plugin<Project> {
    internal lateinit var config: Config

    override fun apply(project: Project) {
        config = project.extensions.create("rust", Config::class.java)
        project.afterEvaluate {
            if (config.targets == null) {
                throw GradleException("targets cannot be null")
            }
            if (config.arches == null) {
                throw GradleException("arches cannot be null")
            }
            for (profile in listOf("debug", "release")) {
                val buildTask = project.tasks.maybeCreate("rustBuild${profile.capitalize()}", DefaultTask::class.java).apply {
                    group = TASK_GROUP
                    description = "Build dynamic library in ${profile} mode for all targets"
                }
                for (targetPair in config.targets!!.withIndex()) {
                    val targetName = targetPair.value
                    val targetArch = config.arches!![targetPair.index]
                    val targetBuildTask = project.tasks.maybeCreate("rustBuild${targetArch.capitalize()}${profile.capitalize()}", BuildTask::class.java).apply {
                        group = TASK_GROUP
                        description = "Build dynamic library in ${profile} mode for $targetArch"
                        rootDirRel = File(config.rootDirRel)
                        target = targetName
                        release = profile == "release"
                    }
                    buildTask.dependsOn(targetBuildTask)
                    project.tasks.findByName("preBuild")?.mustRunAfter(targetBuildTask)
                }
            }
        }
    }
}
