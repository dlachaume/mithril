(self.webpackChunk_N_E=self.webpackChunk_N_E||[]).push([[456],{1659:function(e,r,t){Promise.resolve().then(t.bind(t,8031))},8031:function(e,r,t){"use strict";t.r(r),t.d(r,{default:function(){return S}});var i=t(7437),n=t(9376),s=t(2265),o=t(8575),d=t(4442),l=t(9139),a=t(3735),c=t(279),h=t(4402),u=t(4273),x=t(4670),g=t(2003),j=t(3510),m=t(9313),p=t(4403),v=t(9420),f=t(6693);function b(e){let{registrations:r,onClose:t,mode:n}=e,d=(0,o.v9)(e=>(0,j.d1)(e)),[l,a]=(0,s.useState)(void 0),c="out"===n?"danger":"success";function h(e,r){return e.pool_ticker.localeCompare(r.pool_ticker)||e.party_id.localeCompare(r.party_id)}return(0,s.useEffect)(()=>{if(void 0===n)return;let e="";for(let[t,i]of Object.entries(r).reverse()){let r="in"===n?i.in:i.out;if(0!==r.length)for(let i of(e+="Since epoch **#".concat(t,"**:\n"),r.map(e=>{var r,t,i;return{party_id:e.party_id,pool_ticker:null!==(i=null==d?void 0:null===(t=d.pools)||void 0===t?void 0:null===(r=t.find(r=>r.party_id===e.party_id))||void 0===r?void 0:r.pool_ticker)&&void 0!==i?i:""}}).sort(h)))e+="* ".concat(i.party_id),""!==i.pool_ticker&&(e+=" **".concat(i.pool_ticker,"**")),e+="\n"}a(e)},[r,n,d]),(0,i.jsxs)(p.Z,{show:void 0!==n,onHide:function(){t()},size:"xl","aria-labelledby":"contained-modal-title-vcenter",centered:!0,children:[(0,i.jsx)(p.Z.Header,{closeButton:!0,children:(0,i.jsxs)(p.Z.Title,{children:[(0,i.jsx)("i",{className:"bi bi-markdown text-".concat(c)})," Markdown formatted message of"," ","out"===n?"de-registered":"newly registered"," signers"]})}),(0,i.jsx)(p.Z.Body,{children:void 0!==r&&(0,i.jsx)(v.Z,{bg:"light",border:c,children:(0,i.jsx)(v.Z.Body,{children:(0,i.jsx)("pre",{className:"mb-0",children:(0,i.jsx)("code",{children:l})})})})}),(0,i.jsx)(p.Z.Footer,{children:(0,i.jsx)(f.Z,{text:"Copy to clipboard",variant:"primary",textToCopy:l})})]})}var k=t(9697),Z=t(2491),y=t(4076),_=t(413);function w(e){let{onClick:r}=e;return(0,i.jsx)(k.Z,{overlay:(0,i.jsx)(Z.Z,{children:"Markdown formatted code block"}),children:(0,i.jsx)(y.Z,{variant:"light",size:"sm",className:"border-dark",onClick:r,children:(0,i.jsx)("i",{className:"bi bi-markdown"})})})}function N(e){let{mode:r,registrations:t,onMarkdownButtonClick:n}=e,o=e=>"out"===r?e.out:e.in,d=Object.entries(t).reverse().filter(e=>{let[r,t]=e;return o(t).length>0});return(0,i.jsxs)(i.Fragment,{children:[(0,i.jsx)("h3",{children:(0,i.jsxs)(l.Z,{direction:"horizontal",children:[(0,i.jsxs)("div",{children:["out"===r?(0,i.jsxs)(i.Fragment,{children:[(0,i.jsx)("i",{className:"bi bi-box-arrow-left"})," De-registered"]}):(0,i.jsxs)(i.Fragment,{children:[(0,i.jsx)("i",{className:"bi bi-box-arrow-in-right"})," Newly registered"]})," ","Signers"]}),(0,i.jsx)("div",{className:"ms-auto",children:(0,i.jsx)(w,{onClick:n})})]})}),d.map(e=>{let[t,n]=e;return(0,i.jsxs)(s.Fragment,{children:[(0,i.jsxs)("h4",{children:["out"===r?"De-registered":"Registered"," since epoch"," ",(0,i.jsxs)("span",{className:"text-secondary",children:["#",t]})]}),(0,i.jsx)(_.Z,{signers:o(n)})]},t)})]})}function S(){let e=(0,o.I0)(),r=(0,n.useSearchParams)(),[t,p]=(0,s.useState)(!0),[v,f]=(0,s.useState)(void 0),[k,Z]=(0,s.useState)(void 0),[y,_]=(0,s.useState)(void 0),[w,S]=(0,s.useState)(void 0),[C,O]=(0,s.useState)(void 0),[R,E]=(0,s.useState)(void 0);if((0,s.useEffect)(()=>{let t;let i=r.get(g.aggregatorSearchParam);Z(i),(0,d.checkUrl)(i)||(t="invalidAggregatorUrl"),void 0===t?(p(!0),(0,m.fetchEpochSettings)(i).then(e=>{let r=null==e?void 0:e.epoch;if(_(r),r)return Promise.all([(0,m.fetchRegistrations)(i,r),(0,m.fetchRegistrations)(i,r-1),(0,m.fetchRegistrations)(i,r-2),(0,m.fetchRegistrations)(i,r-3)]).then(e=>{let[r,...t]=e;return(0,d.computeInOutRegistrations)(r,...t)})}).then(e=>{S(e),O((0,d.dedupInOutRegistrations)(e))}).then(()=>p(!1)).catch(()=>{_(void 0)}),e((0,j.Q9)(i))):f(t)},[r]),void 0!==v){let e="";return e="invalidAggregatorUrl"===v?"The given aggregator isn't a valid url, please correct it and try again.":"Something went wrong",(0,i.jsxs)(l.Z,{gap:3,children:[(0,i.jsx)("h2",{children:"In/Out Registrations"}),(0,i.jsxs)(a.Z,{variant:"danger",children:[(0,i.jsx)(a.Z.Heading,{children:"Oh snap! You got an error!"}),(0,i.jsx)("p",{children:e})]})]})}return(0,i.jsxs)(l.Z,{gap:3,children:[(0,i.jsx)("h2",{children:"In/Out Registrations"}),t?(0,i.jsx)(c.Z,{animation:"grow"}):(0,i.jsxs)(i.Fragment,{children:[(0,i.jsx)(b,{registrations:C,onClose:function(){E(void 0)},mode:R}),(0,i.jsx)(h.Z,{children:(0,i.jsx)(u.Z,{children:(0,i.jsxs)("tbody",{children:[(0,i.jsxs)("tr",{children:[(0,i.jsx)("td",{children:(0,i.jsx)("strong",{children:"Aggregator:"})}),(0,i.jsx)("td",{children:k})]}),(0,i.jsxs)("tr",{children:[(0,i.jsx)("td",{children:(0,i.jsx)("strong",{children:"Current epoch:"})}),(0,i.jsx)("td",{children:y})]})]})})}),(0,i.jsxs)(h.Z,{children:[(0,i.jsx)(x.Z,{xs:12,sm:12,md:12,lg:6,children:(0,i.jsx)("div",{className:"p-2 mb-2 border border-danger rounded",children:(0,i.jsx)(N,{registrations:C,mode:"out",onMarkdownButtonClick:()=>E("out")})})}),(0,i.jsx)(x.Z,{xs:12,sm:12,md:12,lg:6,children:(0,i.jsx)("div",{className:"p-2 border border-success rounded",children:(0,i.jsx)(N,{registrations:C,mode:"in",onMarkdownButtonClick:()=>E("in")})})})]})]})]})}}},function(e){e.O(0,[855,182,288,413,971,117,744],function(){return e(e.s=1659)}),_N_E=e.O()}]);