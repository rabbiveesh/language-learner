(window["webpackJsonp"]=window["webpackJsonp"]||[]).push([["5b7fe6e8"],{"6ca3":function(e,t,a){"use strict";var n=a("77d0"),s=a.n(n);s.a},"77d0":function(e,t,a){},"8b24":function(e,t,a){"use strict";a.r(t);var n=function(){var e=this,t=e.$createElement,a=e._self._c||t;return a("q-page",{staticClass:"flex flex-center column"},[a("q-tabs",{model:{value:e.tab,callback:function(t){e.tab=t},expression:"tab"}},e._l(e.words,(function(e){return a("q-tab",{key:e,attrs:{label:e,name:e}})})),1),a("q-tab-panels",{attrs:{animated:""},model:{value:e.tab,callback:function(t){e.tab=t},expression:"tab"}},e._l(e.words,(function(e){return a("q-tab-panel",{key:e,staticClass:"fill",attrs:{name:e}},[a("images-tab",{attrs:{word:e}})],1)})),1),a("q-page-sticky",{attrs:{position:"bottom-right",offset:[18,18]}},[a("q-btn",{attrs:{fab:"",icon:"refresh"},on:{click:e.getWords}})],1)],1)},s=[],c=function(){var e=this,t=e.$createElement,a=e._self._c||t;return a("div",e._l(e.images,(function(t,n){return a("selectable-img",{key:n,attrs:{src:t.src,selected:e.selected===n},on:{selected:function(t){return e.toggle(n,t)}}})})),1)},o=[],i=function(){var e=this,t=e.$createElement,a=e._self._c||t;return a("img",{class:["col",{selected:e.selected}],attrs:{src:e.src},on:{click:e.select}})},l=[],r={name:"SelectableImg",props:["src","selected"],data:function(){return{}},methods:{select:function(){this.$emit("selected",!this.selected)}}},d=r,u=(a("6ca3"),a("2877")),b=Object(u["a"])(d,i,l,!1,null,null,null),f=b.exports,m={name:"ImagesTab",components:{SelectableImg:f},props:["word"],data:function(){return{images:[],selected:!1}},methods:{getImages:function(){var e=this;this.$axios.get("/img/".concat(this.word)).then((function(t){var a=t.data;e.images=a})),this.$axios.get("/selection/".concat(this.word)).then((function(t){var a=t.data;console.log(a),e.selected=a}))},toggle:function(e,t){this.selected=!!t&&e,this.$axios.post("/select/img",{selection:this.selected,word:this.word})}},mounted:function(){this.getImages()}},g=m,h=Object(u["a"])(g,c,o,!1,null,null,null),p=h.exports,w={name:"PageIndex",components:{ImagesTab:p},data:function(){return{words:["your","face"],tab:""}},methods:{getWords:function(){var e=this;this.$axios.get("/words").then((function(t){e.words=t.data,e.tab=e.words[0]}))}},mounted:function(){var e=this;this.getWords(),window.addEventListener("unload",(function(){navigator.sendBeacon("".concat(e.$baseURL,"/exit"))}))}},v=w,x=a("eebe"),k=a.n(x),$=a("9989"),_=a("429b"),q=a("7460"),I=a("adad"),Q=a("823b"),y=a("de5e"),T=a("9c40"),P=Object(u["a"])(v,n,s,!1,null,null,null);t["default"]=P.exports;k()(P,"components",{QPage:$["a"],QTabs:_["a"],QTab:q["a"],QTabPanels:I["a"],QTabPanel:Q["a"],QPageSticky:y["a"],QBtn:T["a"]})}}]);