package {{reverse-domain app.domain}}

import java.io.File
import org.apache.tools.ant.taskdefs.condition.Os
import org.gradle.api.DefaultTask
import org.gradle.api.GradleException
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
        val rootDirRel = rootDirRel ?: throw GradleException("rootDirRel cannot be null")
        val target = target ?: throw GradleException("target cannot be null")
        val release = release ?: throw GradleException("release cannot be null")
        val executable = {{executable}};
        val args = listOf({{quote-and-join tauri-binary-args}});
        try {
            project.exec {
                workingDir(File(project.projectDir, rootDirRel.path))
                executable(executable)
                args(args)
                if (project.logger.isEnabled(LogLevel.DEBUG)) {
                    args("-vv")
                } else if (project.logger.isEnabled(LogLevel.INFO)) {
                    args("-v")
                }
                if (release) {
                    args("--release")
                }
                args(listOf("--target", target))
            }.assertNormalExitValue()
        } catch (e: Exception){
            if (Os.isFamily(Os.FAMILY_WINDOWS)) {
                project.exec {
                    workingDir(File(project.projectDir, rootDirRel.path))
                    executable("$executable.cmd")
                    args(args)
                    if (project.logger.isEnabled(LogLevel.DEBUG)) {
                        args("-vv")
                    } else if (project.logger.isEnabled(LogLevel.INFO)) {
                        args("-v")
                    }
                    if (release) {
                        args("--release")
                    }
                    args(listOf("--target", target))
                }.assertNormalExitValue()
            } else {
                throw e;
            }
        }
    }
}

