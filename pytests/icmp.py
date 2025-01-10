from scapy.all import Ether, IP, ICMP, sendp

# 构建 Ethernet 帧
ether = Ether(dst="ff:ff:ff:ff:ff:ff")  # 广播地址
# 构建 IP 层
ip = IP(src="10.0.0.1", dst="10.0.0.2")
# 构建 ICMP 层
icmp = ICMP(type=8)  # Echo Request

# 组合成完整的数据包
packet = ether / ip / icmp

# 通过指定接口发送数据包
sendp(packet, iface="tun0")