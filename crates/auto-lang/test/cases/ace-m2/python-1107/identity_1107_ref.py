# Independent native-Python reference for identity_1107.at (no ACE involved).
import torch

torch.manual_seed(0)
t = torch.ones(2, 1)
t.fill_(3.0)
print(t.mul(1.0).sum().item())
base = id(t)
print(1 if id(t) - base == 0 else 0)
u = t.mul(1.0)
print(1 if id(u) - base != 0 else 0)
print(u.add(u).sum().item())
print(t.sum().item())