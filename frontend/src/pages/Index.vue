<template>
  <q-page class="flex flex-center column">
    <q-btn label="Reload" @click="getWords" />
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
  methods: {
    getWords () {
      this.$axios.get('http://localhost:9099/words')
        .then((response) => {
          this.words = response.data
        })
    }
  },
  mounted () {
    this.getWords()
  }
}
</script>
