#include <errno.h>
#include <fcntl.h>
#include <linux/if_ether.h>
#include <linux/if_tun.h>
#include <linux/in.h>
#include <net/if.h>
#include <string.h>
#include <strings.h>
#include <sys/ioctl.h>

/** \return `errno` on error, 0 on success */
int setpersist_tap(int fd) {
  /* if EBUSY, we donot set persist to tap */
  if (ioctl(fd, TUNSETPERSIST, 1) < 0) {
    return errno;
  }
  return 0;
}

int getmtu_tap(int skfd, const char *name, int *mtu) {
  struct ifreq ifr;
  bzero(&ifr, sizeof(ifr));
  strncpy(ifr.ifr_name, name, IFNAMSIZ);
  /* get net order hardware address */
  if (ioctl(skfd, SIOCGIFMTU, (void *)&ifr) < 0) {
    return errno;
  }
  *mtu = ifr.ifr_mtu;
  return 0;
}

/**
 * \return `errno` on error, 0 on success
 * \param name: interface name
 * \param ipaddr: ipv4 address
 */
int setipaddr_tap(int skfd, const char *name, unsigned int ipaddr) {
  struct ifreq ifr;
  bzero(&ifr, sizeof(ifr));
  strncpy(ifr.ifr_name, name, IFNAMSIZ);
  struct sockaddr_in *saddr = (struct sockaddr_in *)&ifr.ifr_addr;
  saddr->sin_family = AF_INET;
  saddr->sin_addr.s_addr = ipaddr;
  if (ioctl(skfd, SIOCSIFADDR, (void *)&ifr) < 0) {
    return errno;
  }
  return 0;
}

/**
 * \return `errno` on error, 0 on success
 * \param name: interface name
 * \param ipaddr: ipv4 address
 */
int getipaddr_tap(int skfd, const char *name, unsigned int *ipaddr) {
  struct ifreq ifr;
  bzero(&ifr, sizeof(ifr));
  strncpy(ifr.ifr_name, name, IFNAMSIZ);
  if (ioctl(skfd, SIOCGIFADDR, (void *)&ifr) < 0) {
    return errno;
  }
  struct sockaddr_in *saddr = (struct sockaddr_in *)&ifr.ifr_addr;
  *ipaddr = saddr->sin_addr.s_addr;
  return 0;
}

int setnetmask_tap(int skfd, const char *name, unsigned int netmask) {
  struct ifreq ifr;
  bzero(&ifr, sizeof(ifr));

  strncpy(ifr.ifr_name, name, IFNAMSIZ);
  struct sockaddr_in *saddr = (struct sockaddr_in *)&ifr.ifr_netmask;
  saddr->sin_family = AF_INET;
  saddr->sin_addr.s_addr = netmask;
  if (ioctl(skfd, SIOCSIFNETMASK, (void *)&ifr) < 0) {
    return errno;
  }
  return 0;
}

static int setflags_tap(int skfd, const char *name, unsigned short flags, int set) {
  struct ifreq ifr;
  bzero(&ifr, sizeof(ifr));
  strncpy(ifr.ifr_name, name, IFNAMSIZ);
  /* get original flags */
  if (ioctl(skfd, SIOCGIFFLAGS, (void *)&ifr) < 0) {
    return errno;
  }
  /* set new flags */
  if (set) {
    ifr.ifr_flags |= flags;
  } else {
    ifr.ifr_flags &= ~flags & 0xffff;
  }
  if (ioctl(skfd, SIOCSIFFLAGS, (void *)&ifr) < 0) {
    return errno;
  }
  return 0;
}

int setup_tap(int skfd, const char *name) { return setflags_tap(skfd, name, IFF_UP | IFF_RUNNING, 1); }

/** \brief get hardware address */
int gethwaddr_tap(int tapfd, unsigned char *ha) {
  struct ifreq ifr;
  bzero(&ifr, sizeof(ifr));
  /* get net order hardware address */
  if (ioctl(tapfd, SIOCGIFHWADDR, (void *)&ifr) < 0) {
    return errno;
  }
  memcpy(ha, ifr.ifr_hwaddr.sa_data, ETH_ALEN);
  return 0;
}

/**
 * \return `errno` on error, 0 on success
 * \param name return the interface name
 */
int getname_tap(int tapfd, char *name) {
  struct ifreq ifr;
  bzero(&ifr, sizeof(ifr));
  if (ioctl(tapfd, TUNGETIFF, (void *)&ifr) < 0) {
    return errno;
  }
  strncpy(name, ifr.ifr_name, IFNAMSIZ);
  return 0;
}

/**
 * \return `errno` on error, 0 on success
 * \param skfd: return the socket fd
 */
int set_tap(int *skfd) {
  if (skfd == NULL) {
    return -1;
  }
  *skfd = socket(PF_INET, SOCK_DGRAM, IPPROTO_IP); // udp
  if (*skfd < 0) {
    return errno;
  }
  return 0;
}

/** \return `errno` on error, 0 on success */
int set_tap_if(int fd, const char *name) {
  struct ifreq ifr;
  bzero(&ifr, sizeof(ifr));
  ifr.ifr_flags = IFF_TAP | IFF_NO_PI /*without packet info*/;
  strncpy(ifr.ifr_name, name, IFNAMSIZ);
  if (ioctl(fd, TUNSETIFF, (void *)&ifr) < 0) {
    return errno;
  }
  return 0;
}
