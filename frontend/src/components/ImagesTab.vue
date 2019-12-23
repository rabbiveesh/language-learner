<template>
  <div>
    <img class="col" v-for="img in images" :key="img.src"
      :src="img.src" />
  </div>
</template>

<script>
export default {
  name: 'ImagesTab',
  props: [ 'word' ],
  data () {
    return {
      images: []
    }
  },
  methods: {
    getImages () {
      this.$axios.get(`http://localhost:9099/img/${this.word}`)
        .then(({ data }) => { this.images = data })
    }
  },
  mounted () {
    this.getImages()
  }
}
</script>
