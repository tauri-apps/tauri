<template>
  <hero>
    <q-markdown>
You can add markdown to your page by surrounding it with `q-markdown` tag.
Be aware, markdown is sensitive to being on left side, otherwise will wrap it in blocks (indented).
    </q-markdown>

    <q-markdown>
This is an exampe of a title. It calls outside of the markdown, so be to register the label in the TOC below.
    </q-markdown>

    <example-title title="Basic" />
    <example-card title="Tauri - Basic" name="TauriBasic" :tag-parts="getTagParts(require('!!raw-loader!../examples/TauriBasic.vue').default)" />
    <example-card title="Tauri - Advanced" name="TauriAdvanced" :tag-parts="getTagParts(require('!!raw-loader!../examples/TauriAdvanced.vue').default)" />

  </hero>
</template>

<script>
import Hero from '../components/Hero'
import ExampleTitle from '../components/ExampleTitle'
import ExampleCard from '../components/ExampleCard'
import { slugify } from 'assets/page-utils'
import getTagParts from '@quasar/quasar-app-extension-qmarkdown/src/lib/getTagParts'

export default {
  name: 'Examples',

  components: {
    Hero,
    ExampleTitle,
    ExampleCard
  },

  data () {
    return {
      tempToc: []
    }
  },

  mounted () {
    this.toc = []
    this.tempToc = []

    // example of top-level toc
    this.addToToc('Basic')

    // example of second-level toc
    this.addToToc('Tauri - Basic', 2)

    this.addToToc('Tauri - Advanced', 2)

    // add the toc to right drawer
    this.toc = this.tempToc
  },

  computed: {
    toc:
    {
      get () {
        return this.$store.state.common.toc
      },
      set (toc) {
        this.$store.commit('common/toc', toc)
      }
    }
  },

  methods: {
    getTagParts,
    addToToc (name, level = 1) {
      const slug = slugify(name)
      this.tempToc.push({
        children: [],
        id: slug,
        label: name,
        level: level
      })
    }
  }
}
</script>

<style>
</style>
