<template>
  <q-page class="flex flex-center column">
    <h3 class="row text-center">{{ msg }}</h3>
    <div class="row">
      <q-btn @click="eventToRust()">SEND MSG</q-btn>
    </div>
  </q-page>
</template>

<script>
import { uid } from 'quasar'

export default {
  name: 'HelloWorld',
  data () {
    return {
      msg: 'waiting for rust'
    }
  },
  mounted () {
    console.log(window.tauri)
    window.tauri.invoke({
      cmd: 'init'
    })
    window.tauri.listen('reply', res => {
      this.msg = res.payload.msg
    }, false)
  },
  methods: {
    // set up an event listener
    eventToRust () {
      console.log('event')
      window.tauri.emit('hello', uid())
    }
  }
}
</script>
