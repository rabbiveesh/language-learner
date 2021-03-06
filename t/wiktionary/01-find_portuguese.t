use Language::Learner::Test types => 1;
use Language::Learner::Wiktionary;

use Mojo::DOM;

my $assets = curfile->dirname->child('pages');

my $page = $assets->child('feira')->slurp;
ok my $wiki = Language::Learner::Wiktionary->new( 
  dom => $page, language => 'Portuguese' ), 
'we can construct our object';

my @lines = split /\n/, $page;
sub get_dom (@rows) { Mojo::DOM->new(join "\n", @lines[@rows], '' ) }
my $ex_portuguese = get_dom 167..230;
my $ex_galician   = get_dom 104..137;


isa_ok $wiki->dom, ["Mojo::DOM"], 'the dom coerces';

is $wiki->result,
   $ex_portuguese->to_string, 'parsed correctly at end of file';

my $txt = $assets->child('feira.txt')->slurp;
my @md_lines = split /\n/, $txt;
my $plain = join "\n", @md_lines[165..211];

is $wiki->plaintext, $plain, 'converts as expected to plaintext';




ok $wiki->language('Galician'), 'set the language to Portuguese';
is $wiki->result,
   $ex_galician->to_string, 'parsed correctly in mid file';


done_testing;
