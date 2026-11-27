// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.plugin

import androidx.test.ext.junit.runners.AndroidJUnit4
import com.fasterxml.jackson.annotation.JsonAutoDetect
import com.fasterxml.jackson.annotation.PropertyAccessor
import com.fasterxml.jackson.databind.DeserializationFeature
import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.databind.exc.MismatchedInputException
import com.fasterxml.jackson.datatype.jsonorg.JsonOrgModule
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith

class Holder {
  var obj: JSObject? = null
  var arr: JSArray? = null
}

@RunWith(AndroidJUnit4::class)
class JsonOrgSerializationTest {
  private val json = """{"obj":{"nested":{"items":[1,true,null,"text"]},"flag":false},"arr":[{"a":1},[2,3],"x"]}"""

  // configured like PluginManager's mapper, and a bare one (what broadcast receivers create)
  private val mappers = listOf(
    ObjectMapper()
      .disable(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES)
      .enable(DeserializationFeature.FAIL_ON_NULL_FOR_PRIMITIVES)
      .setVisibility(PropertyAccessor.FIELD, JsonAutoDetect.Visibility.ANY)
      .registerModule(JsonOrgModule()),
    ObjectMapper(),
  )

  @Test
  fun deserializesNestedFields() {
    for (mapper in mappers) {
      val holder = mapper.readValue(json, Holder::class.java)
      val obj = holder.obj!!
      assertFalse(obj.getBoolean("flag"))
      assertEquals("text", obj.getJSONObject("nested").getJSONArray("items").getString(3))
      val arr = holder.arr!!
      assertEquals(3, arr.length())
      assertEquals(1, arr.getJSONObject(0).getInt("a"))
      assertEquals(3, arr.getJSONArray(1).getInt(1))
    }
  }

  @Test
  fun serializesAsPlainJson() {
    for (mapper in mappers) {
      val holder = mapper.readValue(json, Holder::class.java)
      assertEquals(mapper.readTree(json), mapper.readTree(mapper.writeValueAsString(holder)))
      assertEquals(mapper.readTree(json).get("obj"), mapper.readTree(mapper.writeValueAsString(holder.obj)))
    }
  }

  @Test
  fun keepsNullsAndRejectsTheWrongShape() {
    for (mapper in mappers) {
      val holder = mapper.readValue("""{"obj":null,"arr":null}""", Holder::class.java)
      assertNull(holder.obj)
      assertNull(holder.arr)
      assertThrows(MismatchedInputException::class.java) {
        mapper.readValue("""{"obj":[1]}""", Holder::class.java)
      }
      assertThrows(MismatchedInputException::class.java) {
        mapper.readValue("""{"arr":{}}""", Holder::class.java)
      }
    }
  }
}
