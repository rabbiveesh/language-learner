(window["webpackJsonp"]=window["webpackJsonp"]||[]).push([["5b7fe6e8"],{"6ca3":function(t,e,a){"use strict";var n=a("77d0"),s=a.n(n);s.a},"77d0":function(t,e,a){},"8b24":function(t,e,a){"use strict";a.r(e);var n=function(){var t=this,e=t.$createElement,a=t._self._c||e;return a("q-page",{staticClass:"flex flex-center column justify-between"},[a("q-tabs",{staticClass:"col-1 row",attrs:{align:"center"},model:{value:t.tab,callback:function(e){t.tab=e},expression:"tab"}},t._l(t.words,(function(t){return a("q-tab",{key:t,attrs:{label:t,name:t}})})),1),a("q-tab-panels",{staticClass:"col-11",attrs:{animated:""},model:{value:t.tab,callback:function(e){t.tab=e},expression:"tab"}},t._l(t.words,(function(t){return a("q-tab-panel",{key:t,staticClass:"fill",attrs:{name:t}},[a("images-tab",{attrs:{word:t}})],1)})),1),a("q-page-sticky",{attrs:{position:"bottom-right",offset:[18,18]}},[a("q-btn",{attrs:{color:"primary",fab:"",icon:"keyboard_arrow_up"},on:{click:t.getWords}})],1)],1)},s=[],c=function(){var t=this,e=t.$createElement,a=t._self._c||e;return a("div",t._l(t.images,(function(e,n){return a("selectable-img",{key:n,attrs:{src:e.src,text:e.text,selected:t.selected===n},on:{selected:function(e){return t.toggle(n,e)}}})})),1)},o=[],i=function(){var t=this,e=t.$createElement,a=t._self._c||e;return a("div",{staticClass:"column inline-block"},[a("img",{class:{selected:t.selected,"q-ma-md":t.selected},attrs:{src:t.src},on:{click:t.select}}),t.selected?a("span",{domProps:{innerHTML:t._s(t.text)}}):t._e()])},l=[],r={name:"SelectableImg",props:["src","selected","text"],data:function(){return{}},methods:{select:function(){this.$emit("selected",!this.selected)}}},d=r,u=(a("6ca3"),a("2877")),b=Object(u["a"])(d,i,l,!1,null,null,null),m=b.exports,f={name:"ImagesTab",components:{SelectableImg:m},props:["word"],data:function(){return{images:[],selected:!1}},methods:{getImages:function(){var t=this;this.$axios.get("/img/".concat(this.word)).then((function(e){var a=e.data;t.images=a})),this.$axios.get("/selection/".concat(this.word)).then((function(e){var a=e.data;console.log(a),t.selected=a}))},toggle:function(t,e){this.selected=!!e&&t,this.$axios.post("/select/img",{selection:this.selected,word:this.word})}},mounted:function(){this.getImages()}},g=f,h=Object(u["a"])(g,c,o,!1,null,null,null),p=h.exports,w={name:"PageIndex",components:{ImagesTab:p},data:function(){return{words:["your","face"],tab:""}},methods:{getWords:function(){var t=this;this.$axios.get("/words").then((function(e){t.words=e.data,t.tab=t.words[0]}))}},mounted:function(){var t=this;this.getWords(),window.addEventListener("unload",(function(){navigator.sendBeacon("".concat(t.$baseURL,"/exit"))}))}},x=w,v=a("eebe"),k=a.n(v),_=a("9989"),y=a("429b"),$=a("7460"),q=a("adad"),I=a("823b"),Q=a("de5e"),T=a("9c40"),P=Object(u["a"])(x,n,s,!1,null,null,null);e["default"]=P.exports;k()(P,"components",{QPage:_["a"],QTabs:y["a"],QTab:$["a"],QTabPanels:q["a"],QTabPanel:I["a"],QPageSticky:Q["a"],QBtn:T["a"]})}}]);