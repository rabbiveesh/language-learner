package Language::Learner::Types;

use Type::Library -base,
  -declare => qw/ DOM /;
use Type::Utils -all;
use Types::Standard 'slurpy';
BEGIN { extends 'Types::Standard' }

use Mojo::DOM;

declare DOM, as InstanceOf['Mojo::DOM'];

coerce DOM, from Str, via {
  Mojo::DOM->new($_)
};

coerce DOM, from InstanceOf['Mojo::Transaction'], via {
  $_->dom
};



9002
