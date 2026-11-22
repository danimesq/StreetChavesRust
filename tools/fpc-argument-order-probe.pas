program FpcArgumentOrderProbe;

{$mode delphi}{$H+}

var
  OrdemTeste: string;

function MarcaB(C: Char): Byte;
begin
  OrdemTeste := OrdemTeste + C;
  Result := 0;
end;

function MarcaD(C: Char): Double;
begin
  OrdemTeste := OrdemTeste + C;
  Result := Length(OrdemTeste);
end;

procedure TesteArgs(Tipo: Byte; const Texto: string; Dono: Byte; X, Y: Double);
begin
  { The procedure body intentionally does not inspect or mutate the arguments. }
end;

begin
  OrdemTeste := '';
  { Keep this identical to ChavesRoteiro.pas initialization probe. }
  TesteArgs(MarcaB('1'), '', MarcaB('3'), MarcaD('4'), MarcaD('5'));

  if (Pos('5', OrdemTeste) > 0) and
     (Pos('4', OrdemTeste) > 0) and
     (Pos('5', OrdemTeste) < Pos('4', OrdemTeste)) then
    WriteLn('argument-order=right-to-left')
  else
    WriteLn('argument-order=left-to-right');

  WriteLn('evaluation-trace=' + OrdemTeste);
end.
