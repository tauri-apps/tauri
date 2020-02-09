<template>
  <q-page class="flex flex-center column">
    <h4 class="row text-center">Node: {{ nodeMsg }}</h4>
    <h3 class="row text-center">{{ msg }}</h3>
    <div class="row">
      <q-btn @click="eventToRust()">SEND MSG</q-btn>
    </div>
  </q-page>
</template>

<script>
import { uid } from 'quasar'
import tauri from 'tauri/api'

export default {
  name: 'HelloWorld',
  data () {
    return {
      msg: 'waiting for rust',
      nodeMsg: 'waiting'
    }
  },
  mounted () {
    tauri.listen('reply', res => {
      this.msg = res.payload.msg
    })
    tauri.listen('node', res => {
      this.nodeMsg = res.payload
    })
  },
  methods: {
    // set up an event listener
    eventToRust () {
      tauri.emit('hello', uid())
    }
  }
}
</script>
