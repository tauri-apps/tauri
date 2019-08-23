<template>
  <div class="q-pa-md">
    <p>You can put any regular Vue code (SFC) into one of these files.</p>
    <p>It will be displayed as an example.</p>
    <p>template, script, and css will be separated out into 3 parts and each will have a tab.</p>
  </div>
</template>

<script>
export default {
  data () {
    return {
      var1: 10,
      var2: 'string'
    }
  }
}
</script>

<style lang="stylus">
.my-class
  background-color black
</style>
