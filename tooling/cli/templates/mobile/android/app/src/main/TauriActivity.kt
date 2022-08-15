package {{reverse-domain app.domain}}.{{snake-case app.name}}

import android.os.Bundle
import androidx.appcompat.app.AppCompatActivity

abstract class TauriActivity : AppCompatActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        create(this)
    }

    override fun onStart() {
        super.onStart()
        start()
    }

    override fun onResume() {
        super.onResume()
        resume()
    }

    override fun onPause() {
        super.onPause()
        pause()
    }

    override fun onStop() {
        super.onStop()
        stop()
    }

    override fun onWindowFocusChanged(hasFocus: Boolean) {
        super.onWindowFocusChanged(hasFocus)
        focus(hasFocus)
    }

    override fun onSaveInstanceState(outState: Bundle) {
        super.onSaveInstanceState(outState)
        save()
    }

    override fun onDestroy() {
        super.onDestroy()
        destroy()
    }

    override fun onLowMemory() {
        super.onLowMemory()
        memory()
    }

    fun getAppClass(name: String): Class<*> {
        return Class.forName(name)
    }

    companion object {
        init {
            System.loadLibrary("{{snake-case app.name}}")
        }
    }

    private external fun create(activity: TauriActivity)
    private external fun start()
    private external fun resume()
    private external fun pause()
    private external fun stop()
    private external fun save()
    private external fun destroy()
    private external fun memory()
    private external fun focus(focus: Boolean)
}
