(function() {
    var type_impls = Object.fromEntries([["mithril_common",[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Clone-for-StmClerk%3CD%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/mithril_stm/stm.rs.html#191\">source</a><a href=\"#impl-Clone-for-StmClerk%3CD%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;D&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"mithril_stm/stm/struct.StmClerk.html\" title=\"struct mithril_stm::stm::StmClerk\">StmClerk</a>&lt;D&gt;<div class=\"where\">where\n    D: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + Digest,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.clone\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/mithril_stm/stm.rs.html#191\">source</a><a href=\"#method.clone\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.82.0/core/clone/trait.Clone.html#tymethod.clone\" class=\"fn\">clone</a>(&amp;self) -&gt; <a class=\"struct\" href=\"mithril_stm/stm/struct.StmClerk.html\" title=\"struct mithril_stm::stm::StmClerk\">StmClerk</a>&lt;D&gt;</h4></section></summary><div class='docblock'>Returns a copy of the value. <a href=\"https://doc.rust-lang.org/1.82.0/core/clone/trait.Clone.html#tymethod.clone\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.clone_from\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.0.0\">1.0.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/1.82.0/src/core/clone.rs.html#174\">source</a></span><a href=\"#method.clone_from\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.82.0/core/clone/trait.Clone.html#method.clone_from\" class=\"fn\">clone_from</a>(&amp;mut self, source: &amp;Self)</h4></section></summary><div class='docblock'>Performs copy-assignment from <code>source</code>. <a href=\"https://doc.rust-lang.org/1.82.0/core/clone/trait.Clone.html#method.clone_from\">Read more</a></div></details></div></details>","Clone","mithril_common::crypto_helper::types::alias::ProtocolClerk"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Debug-for-StmClerk%3CD%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/mithril_stm/stm.rs.html#191\">source</a><a href=\"#impl-Debug-for-StmClerk%3CD%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;D&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"mithril_stm/stm/struct.StmClerk.html\" title=\"struct mithril_stm::stm::StmClerk\">StmClerk</a>&lt;D&gt;<div class=\"where\">where\n    D: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + Digest,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.fmt\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/mithril_stm/stm.rs.html#191\">source</a><a href=\"#method.fmt\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.82.0/core/fmt/trait.Debug.html#tymethod.fmt\" class=\"fn\">fmt</a>(&amp;self, f: &amp;mut <a class=\"struct\" href=\"https://doc.rust-lang.org/1.82.0/core/fmt/struct.Formatter.html\" title=\"struct core::fmt::Formatter\">Formatter</a>&lt;'_&gt;) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.82.0/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.82.0/std/primitive.unit.html\">()</a>, <a class=\"struct\" href=\"https://doc.rust-lang.org/1.82.0/core/fmt/struct.Error.html\" title=\"struct core::fmt::Error\">Error</a>&gt;</h4></section></summary><div class='docblock'>Formats the value using the given formatter. <a href=\"https://doc.rust-lang.org/1.82.0/core/fmt/trait.Debug.html#tymethod.fmt\">Read more</a></div></details></div></details>","Debug","mithril_common::crypto_helper::types::alias::ProtocolClerk"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-StmClerk%3CD%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/mithril_stm/stm.rs.html#496\">source</a><a href=\"#impl-StmClerk%3CD%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;D&gt; <a class=\"struct\" href=\"mithril_stm/stm/struct.StmClerk.html\" title=\"struct mithril_stm::stm::StmClerk\">StmClerk</a>&lt;D&gt;<div class=\"where\">where\n    D: Digest + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + FixedOutput,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.from_registration\" class=\"method\"><a class=\"src rightside\" href=\"src/mithril_stm/stm.rs.html#498\">source</a><h4 class=\"code-header\">pub fn <a href=\"mithril_stm/stm/struct.StmClerk.html#tymethod.from_registration\" class=\"fn\">from_registration</a>(\n    params: &amp;<a class=\"struct\" href=\"mithril_stm/stm/struct.StmParameters.html\" title=\"struct mithril_stm::stm::StmParameters\">StmParameters</a>,\n    closed_reg: &amp;<a class=\"struct\" href=\"mithril_stm/key_reg/struct.ClosedKeyReg.html\" title=\"struct mithril_stm::key_reg::ClosedKeyReg\">ClosedKeyReg</a>&lt;D&gt;,\n) -&gt; <a class=\"struct\" href=\"mithril_stm/stm/struct.StmClerk.html\" title=\"struct mithril_stm::stm::StmClerk\">StmClerk</a>&lt;D&gt;</h4></section></summary><div class=\"docblock\"><p>Create a new <code>Clerk</code> from a closed registration instance.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.from_signer\" class=\"method\"><a class=\"src rightside\" href=\"src/mithril_stm/stm.rs.html#506\">source</a><h4 class=\"code-header\">pub fn <a href=\"mithril_stm/stm/struct.StmClerk.html#tymethod.from_signer\" class=\"fn\">from_signer</a>(signer: &amp;<a class=\"struct\" href=\"mithril_stm/stm/struct.StmSigner.html\" title=\"struct mithril_stm::stm::StmSigner\">StmSigner</a>&lt;D&gt;) -&gt; <a class=\"struct\" href=\"mithril_stm/stm/struct.StmClerk.html\" title=\"struct mithril_stm::stm::StmClerk\">StmClerk</a>&lt;D&gt;</h4></section></summary><div class=\"docblock\"><p>Create a Clerk from a signer.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.aggregate\" class=\"method\"><a class=\"src rightside\" href=\"src/mithril_stm/stm.rs.html#524-528\">source</a><h4 class=\"code-header\">pub fn <a href=\"mithril_stm/stm/struct.StmClerk.html#tymethod.aggregate\" class=\"fn\">aggregate</a>(\n    &amp;self,\n    sigs: &amp;[<a class=\"struct\" href=\"mithril_stm/stm/struct.StmSig.html\" title=\"struct mithril_stm::stm::StmSig\">StmSig</a>],\n    msg: &amp;[<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.82.0/std/primitive.u8.html\">u8</a>],\n) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.82.0/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"struct\" href=\"mithril_stm/stm/struct.StmAggrSig.html\" title=\"struct mithril_stm::stm::StmAggrSig\">StmAggrSig</a>&lt;D&gt;, <a class=\"enum\" href=\"mithril_stm/error/enum.AggregationError.html\" title=\"enum mithril_stm::error::AggregationError\">AggregationError</a>&gt;</h4></section></summary><div class=\"docblock\"><p>Aggregate a set of signatures for their corresponding indices.</p>\n<p>This function first deduplicates the repeated signatures, and if there are enough signatures, it collects the merkle tree indexes of unique signatures.\nThe list of merkle tree indexes is used to create a batch proof, to prove that all signatures are from eligible signers.</p>\n<p>It returns an instance of <code>StmAggrSig</code>.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.compute_avk\" class=\"method\"><a class=\"src rightside\" href=\"src/mithril_stm/stm.rs.html#562\">source</a><h4 class=\"code-header\">pub fn <a href=\"mithril_stm/stm/struct.StmClerk.html#tymethod.compute_avk\" class=\"fn\">compute_avk</a>(&amp;self) -&gt; <a class=\"struct\" href=\"mithril_stm/stm/struct.StmAggrVerificationKey.html\" title=\"struct mithril_stm::stm::StmAggrVerificationKey\">StmAggrVerificationKey</a>&lt;D&gt;</h4></section></summary><div class=\"docblock\"><p>Compute the <code>StmAggrVerificationKey</code> related to the used registration.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.get_reg_party\" class=\"method\"><a class=\"src rightside\" href=\"src/mithril_stm/stm.rs.html#567\">source</a><h4 class=\"code-header\">pub fn <a href=\"mithril_stm/stm/struct.StmClerk.html#tymethod.get_reg_party\" class=\"fn\">get_reg_party</a>(&amp;self, party_index: &amp;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.82.0/std/primitive.u64.html\">u64</a>) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.82.0/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;(<a class=\"struct\" href=\"mithril_stm/multi_sig/struct.VerificationKey.html\" title=\"struct mithril_stm::multi_sig::VerificationKey\">VerificationKey</a>, <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.82.0/std/primitive.u64.html\">u64</a>)&gt;</h4></section></summary><div class=\"docblock\"><p>Get the (VK, stake) of a party given its index.</p>\n</div></details></div></details>",0,"mithril_common::crypto_helper::types::alias::ProtocolClerk"]]]]);
    if (window.register_type_impls) {
        window.register_type_impls(type_impls);
    } else {
        window.pending_type_impls = type_impls;
    }
})()
//{"start":55,"fragment_lengths":[10106]}