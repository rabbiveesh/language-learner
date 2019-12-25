import Vue from 'vue'
import axios from 'axios'

const baseURL = process.env.NODE_ENV === 'development'
  ? 'http://localhost:9099'
  : window.location.protocol +
  '//' + window.location.hostname +
  ':' + window.location.port

const axiosInstance = axios.create({ baseURL })
Vue.prototype.$axios = axiosInstance
Vue.prototype.$baseURL = baseURL
