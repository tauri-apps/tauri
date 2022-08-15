package {{reverse-domain app.domain}}

import com.android.build.gradle.*
import java.io.File
import org.gradle.api.DefaultTask
import org.gradle.api.GradleException
import org.gradle.api.Project
import org.gradle.api.logging.LogLevel
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.InputDirectory
import org.gradle.api.tasks.PathSensitive
import org.gradle.api.tasks.PathSensitivity
import org.gradle.api.tasks.TaskAction

open class BuildTask : DefaultTask() {
    @InputDirectory
    @PathSensitive(PathSensitivity.RELATIVE)
    var rootDirRel: File? = null
    @Input
    var target: String? = null
    @Input
    var release: Boolean? = null

    @TaskAction
    fun build() {
        val rootDirRel = rootDirRel
        if (rootDirRel == null) {
            throw GradleException("rootDirRel cannot be null")
        }
        val target = target
        if (target == null) {
            throw GradleException("target cannot be null")
        }
        val release = release
        if (release == null) {
            throw GradleException("release cannot be null")
        }
        project.exec {
            workingDir(File(project.getProjectDir(), rootDirRel.getPath()))
            executable("cargo")
            args(listOf("android", "build"))
            if (project.logger.isEnabled(LogLevel.DEBUG)) {
                args("-vv")
            } else if (project.logger.isEnabled(LogLevel.INFO)) {
                args("-v")
            }
            if (release) {
                args("--release")
            }
            args("${target}")
        }.assertNormalExitValue()
    }
}

