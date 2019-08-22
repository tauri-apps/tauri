<template>
  <h1 :id="slugifiedTitle" class="q-markdown--heading-h1 q-markdown--title-heavy example-title" @click="copyHeading(slugifiedTitle)">{{ title }}</h1>
</template>

<script>
import { copyHeading, slugify } from 'assets/page-utils'

export default {
  name: 'ExampleTitle',

  props: {
    title: {
      type: String,
      required: true
    }
  },

  computed: {
    slugifiedTitle () {
      return slugify(this.title)
    }
  },

  methods: {
    copyHeading
  }
}
</script>
