<template>
  <div>
    <selectable-img v-for="(img, idx) in images" :key="idx"
      :src="img.src" :selected="selected === idx"
      @selected="v => toggle(idx,v)" />
  </div>
</template>

<script>
import SelectableImg from './SelectableImg.vue'

export default {
  name: 'ImagesTab',
  components: { SelectableImg },
  props: [ 'word' ],
  data () {
    return {
      images: [],
      selected: false
    }
  },
  methods: {
    getImages () {
      this.$axios.get(`/img/${this.word}`)
        .then(({ data }) => { this.images = data })
    },
    toggle (idx, v) {
      if (v) {
        this.selected = idx
      } else {
        this.selected = false
      }
      this.$axios.post('/select/img',
        { selection: this.selected, word: this.word })
    }
  },
  mounted () {
    this.getImages()
  }
}
</script>
