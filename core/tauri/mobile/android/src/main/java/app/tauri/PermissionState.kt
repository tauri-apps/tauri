package app.tauri

import java.util.*

enum class PermissionState(private val state: String) {
  GRANTED("granted"), DENIED("denied"), PROMPT("prompt"), PROMPT_WITH_RATIONALE("prompt-with-rationale");

  override fun toString(): String {
    return state
  }

  companion object {
    fun byState(state: String): PermissionState {
      var state = state
      state = state.uppercase(Locale.ROOT).replace('-', '_')
      return valueOf(state)
    }
  }
}
