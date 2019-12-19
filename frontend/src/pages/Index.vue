<template>
  <q-page class="flex flex-center">
    <div v-for="word in words" :key="word">
      {{ word }}
    </div>
  </q-page>
</template>

<script>
export default {
  name: 'PageIndex',
  data () {
    return {
      words: []
    }
  },
  mounted () {
    this.$axios.get('http://localhost:9099/words')
      .then((response) => {
        this.words = response.data
      })
  }
}
</script>
