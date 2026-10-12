import argparse

from rich.console import Console
from rich.table import Table
from tplinkrouterc6u import Connection

from .core import session

console = Console()

WIFI_BANDS = {
    "host": (Connection.HOST_2G, Connection.HOST_5G),
    "guest": (Connection.GUEST_2G, Connection.GUEST_5G),
    "iot": (Connection.IOT_2G, Connection.IOT_5G),
}


def _band(conn: Connection) -> str:
    return {
        Connection.UNKNOWN: "?",
        Connection.WIRED: "cabo",
    }.get(conn, conn.value.replace("_", " "))


def cmd_nodes(_args):
    with session() as c:
        nodes = c.get_mesh_nodes()
    names = {str(n.macaddr): n.name for n in nodes}
    t = Table(title="Decos")
    for col in ("Nome", "Papel", "IP", "MAC", "Conectado a", "Internet", "Firmware"):
        t.add_column(col)
    for n in sorted(nodes, key=lambda n: n.role != "master"):
        parent = names.get(str(n.parent_macaddr), "-") if n.parent_macaddr else "-"
        t.add_row(n.name, n.role, str(n.ipaddr), str(n.macaddr), parent,
                  n.internet_status, n.firmware_version.split()[0])
    console.print(t)


def cmd_clients(_args):
    with session() as c:
        status = c.get_status()
    t = Table(title=f"Clientes ({len(status.devices)})")
    for col in ("Nome", "IP", "MAC", "Rede", "↓ kB/s", "↑ kB/s"):
        t.add_column(col)
    for d in sorted(status.devices, key=lambda d: d.hostname.lower()):
        t.add_row(d.hostname, str(d.ipaddr), str(d.macaddr), _band(d.type),
                  str(d.down_speed or 0), str(d.up_speed or 0))
    console.print(t)


def cmd_status(_args):
    with session() as c:
        fw = c.get_firmware()
        status = c.get_status()
        nodes = c.get_mesh_nodes()
    console.print(f"[bold]Modelo:[/] Deco {fw.model} v{fw.hardware_version}  "
                  f"[bold]Firmware:[/] {fw.firmware_version}")
    console.print(f"[bold]WAN:[/] {status.wan_ipv4_addr}  gw {status.wan_ipv4_gateway}")
    console.print(f"[bold]Decos:[/] {len(nodes)}  "
                  f"[bold]Clientes:[/] {status.clients_total} "
                  f"(wifi {status.wifi_clients_total}, cabo {status.wired_total}, "
                  f"convidado {status.guest_clients_total})")


def cmd_wifi(args):
    with session() as c:
        for band in WIFI_BANDS[args.network]:
            c.set_wifi(band, args.state == "on")
    console.print(f"Wi-Fi {args.network}: {args.state}")


def cmd_reboot(args):
    if not args.yes:
        console.print("Reinicia TODA a rede mesh. Use --yes para confirmar.")
        return
    with session() as c:
        c.reboot()
    console.print("Reboot enviado.")


def main():
    p = argparse.ArgumentParser(prog="deco", description="Controle de Deco M5 pela rede local")
    sub = p.add_subparsers(required=True)
    sub.add_parser("status", help="resumo da rede").set_defaults(func=cmd_status)
    sub.add_parser("nodes", help="lista os decos").set_defaults(func=cmd_nodes)
    sub.add_parser("clients", help="lista clientes conectados").set_defaults(func=cmd_clients)
    w = sub.add_parser("wifi", help="liga/desliga rede Wi-Fi")
    w.add_argument("network", choices=WIFI_BANDS)
    w.add_argument("state", choices=("on", "off"))
    w.set_defaults(func=cmd_wifi)
    r = sub.add_parser("reboot", help="reinicia a rede mesh")
    r.add_argument("--yes", action="store_true")
    r.set_defaults(func=cmd_reboot)
    args = p.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
