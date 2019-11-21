use Language::Learner::Test types => 1;
use Language::Learner::Wiktionary;
use Pry;


my $dom = curfile->dirname->child(qw/pages feira/)->slurp;
my $wiki = Language::Learner::Wiktionary->new( dom => $dom );

my $ex_portuguese = <<HTML;
<h2><span class="mw-headline" id="Portuguese">Portuguese</span><span class="mw-editsection"><span class="mw-editsection-bracket">[</span><a href="/w/index.php?title=feira&amp;action=edit&amp;section=17" title="Edit section: Portuguese">edit</a><span class="mw-editsection-bracket">]</span></span></h2>
<h3><span class="mw-headline" id="Alternative_forms_3">Alternative forms</span><span class="mw-editsection"><span class="mw-editsection-bracket">[</span><a href="/w/index.php?title=feira&amp;action=edit&amp;section=18" title="Edit section: Alternative forms">edit</a><span class="mw-editsection-bracket">]</span></span></h3>
<ul><li><span class="Latn" lang="pt"><a href="/wiki/feyra#Portuguese" title="feyra">feyra</a></span> <span class="ib-brac qualifier-brac">(</span><span class="ib-content qualifier-content">obsolete</span><span class="ib-brac qualifier-brac">)</span></li>
<li><span class="Latn" lang="pt"><a href="/wiki/f%C3%AAra#Portuguese" title="fêra">fêra</a></span> <span class="ib-brac qualifier-brac">(</span><span class="ib-content qualifier-content">eye dialect<span class="ib-comma qualifier-comma">,</span> Brazil</span><span class="ib-brac qualifier-brac">)</span></li></ul>
<h3><span class="mw-headline" id="Etymology_3">Etymology</span><span class="mw-editsection"><span class="mw-editsection-bracket">[</span><a href="/w/index.php?title=feira&amp;action=edit&amp;section=19" title="Edit section: Etymology">edit</a><span class="mw-editsection-bracket">]</span></span></h3>
<p>From <span class="etyl"><a href="https://en.wikipedia.org/wiki/Galician-Portuguese" class="extiw" title="w:Galician-Portuguese">Old Portuguese</a></span> <i class="Latn mention" lang="roa-opt"><a href="/wiki/feira#Old_Portuguese" title="feira">feira</a></i>, from <span class="etyl"><a href="https://en.wikipedia.org/wiki/Late_Latin" class="extiw" title="w:Late Latin">Late Latin</a></span> <i class="Latn mention" lang="la"><a href="/wiki/feria#Latin" title="feria">feria</a></i> <span class="mention-gloss-paren annotation-paren">(</span><span class="mention-gloss-double-quote">“</span><span class="mention-gloss">festival, holiday</span><span class="mention-gloss-double-quote">”</span><span class="mention-gloss-paren annotation-paren">)</span>, from <span class="etyl"><a href="https://en.wikipedia.org/wiki/Latin" class="extiw" title="w:Latin">Latin</a></span> <i class="Latn mention" lang="la"><a href="/wiki/feriae#Latin" title="feriae">feriae</a></i>, from <span class="etyl"><a href="https://en.wikipedia.org/wiki/Proto-Indo-European_language" class="extiw" title="w:Proto-Indo-European language">Proto-Indo-European</a></span> <i class="Latinx mention" lang="ine-pro"><a href="/w/index.php?title=Reconstruction:Proto-Indo-European/dh%C4%93s-&amp;action=edit&amp;redlink=1" class="new" title="Reconstruction:Proto-Indo-European/dhēs- (page does not exist)">*dhēs-</a></i> <span class="mention-gloss-paren annotation-paren">(</span><span class="mention-gloss-double-quote">“</span><span class="mention-gloss">god, godhead, deity</span><span class="mention-gloss-double-quote">”</span><span class="mention-gloss-paren annotation-paren">)</span>.
</p><p>Cognate with <span class="etyl"><a href="https://en.wikipedia.org/wiki/Galician_language" class="extiw" title="w:Galician language">Galician</a></span> <i class="Latn mention" lang="gl"><a href="/wiki/feira#Galician" title="feira">feira</a></i>, <span class="etyl"><a href="https://en.wikipedia.org/wiki/Spanish_language" class="extiw" title="w:Spanish language">Spanish</a></span> <i class="Latn mention" lang="es"><a href="/wiki/feria#Spanish" title="feria">feria</a></i>, <span class="etyl"><a href="https://en.wikipedia.org/wiki/French_language" class="extiw" title="w:French language">French</a></span> <i class="Latn mention" lang="fr"><a href="/wiki/foire#French" title="foire">foire</a></i>, <span class="etyl"><a href="https://en.wikipedia.org/wiki/Italian_language" class="extiw" title="w:Italian language">Italian</a></span> <i class="Latn mention" lang="it"><a href="/wiki/fiera#Italian" title="fiera">fiera</a></i> and <span class="etyl"><a href="https://en.wikipedia.org/wiki/English_language" class="extiw" title="w:English language">English</a></span> <i class="Latn mention" lang="en"><a href="/wiki/fair#English" title="fair">fair</a></i>.
</p>
<h3><span class="mw-headline" id="Pronunciation_3">Pronunciation</span><span class="mw-editsection"><span class="mw-editsection-bracket">[</span><a href="/w/index.php?title=feira&amp;action=edit&amp;section=20" title="Edit section: Pronunciation">edit</a><span class="mw-editsection-bracket">]</span></span></h3>
<ul><li><span class="ib-brac qualifier-brac">(</span><span class="ib-content qualifier-content"><a href="https://en.wikipedia.org/wiki/European_Portuguese" class="extiw" title="w:European Portuguese">Portugal</a></span><span class="ib-brac qualifier-brac">)</span> <a href="/wiki/Wiktionary:International_Phonetic_Alphabet" title="Wiktionary:International Phonetic Alphabet">IPA</a><sup>(<a href="/wiki/Appendix:Portuguese_pronunciation" title="Appendix:Portuguese pronunciation">key</a>)</sup>:&#32;<span class="IPA">/ˈfɐj.ɾɐ/</span>, <span class="IPA">/ˈfej.ɾɐ/</span></li>
<li><span class="ib-brac qualifier-brac">(</span><span class="ib-content qualifier-content"><a href="https://en.wikipedia.org/wiki/Brazilian_Portuguese" class="extiw" title="w:Brazilian Portuguese">Brazil</a></span><span class="ib-brac qualifier-brac">)</span> <a href="/wiki/Wiktionary:International_Phonetic_Alphabet" title="Wiktionary:International Phonetic Alphabet">IPA</a><sup>(<a href="/wiki/Appendix:Portuguese_pronunciation" title="Appendix:Portuguese pronunciation">key</a>)</sup>:&#32;<span class="IPA">/ˈfej.ɾɐ/</span></li>
<li>Hyphenation: <span class="Latn" lang="pt">fei‧ra</span></li></ul>
<h3><span class="mw-headline" id="Noun_2">Noun</span><span class="mw-editsection"><span class="mw-editsection-bracket">[</span><a href="/w/index.php?title=feira&amp;action=edit&amp;section=21" title="Edit section: Noun">edit</a><span class="mw-editsection-bracket">]</span></span></h3>
<p><strong class="Latn headword" lang="pt">feira</strong> <span class="gender"><abbr title="feminine gender">f</abbr></span> (<i>plural</i> <b class="Latn form-of lang-pt p-form-of" lang="pt"><a href="/wiki/feiras#Portuguese" title="feiras">feiras</a></b>)
</p>
<ol><li><a href="/wiki/market" title="market">market</a></li>
<li><a href="/wiki/fair" title="fair">fair</a></li>
<li><a href="/wiki/confusion" title="confusion">confusion</a></li>
<li>designation for a <a href="/wiki/weekday" title="weekday">weekday</a> in Portuguese: e.g. <span class="Latn" lang="pt"><a href="/wiki/segunda-feira#Portuguese" title="segunda-feira">segunda-feira</a></span> for Monday, <span class="Latn" lang="pt"><a href="/wiki/ter%C3%A7a-feira#Portuguese" title="terça-feira">terça-feira</a></span> for Tuesday, etc.</li></ol>
<h4><span class="mw-headline" id="Synonyms">Synonyms</span><span class="mw-editsection"><span class="mw-editsection-bracket">[</span><a href="/w/index.php?title=feira&amp;action=edit&amp;section=22" title="Edit section: Synonyms">edit</a><span class="mw-editsection-bracket">]</span></span></h4>
<ul><li><span class="ib-brac qualifier-brac">(</span><span class="ib-content qualifier-content">market</span><span class="ib-brac qualifier-brac">)</span><span class="ib-colon sense-qualifier-colon">:</span> <span class="Latn" lang="pt"><a href="/wiki/mercado#Portuguese" title="mercado">mercado</a></span></li></ul>
<h4><span class="mw-headline" id="Derived_terms_3">Derived terms</span><span class="mw-editsection"><span class="mw-editsection-bracket">[</span><a href="/w/index.php?title=feira&amp;action=edit&amp;section=23" title="Edit section: Derived terms">edit</a><span class="mw-editsection-bracket">]</span></span></h4>
<ul><li><span class="Latn" lang="pt"><a href="/wiki/feira_da_ladra#Portuguese" title="feira da ladra">feira da ladra</a></span></li>
<li><span class="Latn" lang="pt"><a href="/wiki/feirinha#Portuguese" title="feirinha">feirinha</a></span></li></ul>
<h4><span class="mw-headline" id="Related_terms">Related terms</span><span class="mw-editsection"><span class="mw-editsection-bracket">[</span><a href="/w/index.php?title=feira&amp;action=edit&amp;section=24" title="Edit section: Related terms">edit</a><span class="mw-editsection-bracket">]</span></span></h4>
<ul><li><span class="Latn" lang="pt"><a href="/wiki/f%C3%A9ria#Portuguese" title="féria">féria</a></span></li>
<li><span class="Latn" lang="pt"><a href="/wiki/teo-#Portuguese" title="teo-">teo-</a></span></li></ul>
HTML


is my $sner = $wiki->language('Portuguese'), $ex_portuguese, 
  'parsed correctly at end of file';

ok $wiki->dom, 'the dom exists';


done_testing;
