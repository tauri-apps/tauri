purge_list=()

install_packages() {
    if grep -i ubuntu /etc/os-release; then
        apt-get update

        for pkg in "${@}"; do
            if ! dpkg -L "${pkg}" >/dev/null 2>/dev/null; then
                apt-get install --assume-yes --no-install-recommends "${pkg}"

                purge_list+=( "${pkg}" )
            fi
        done
    else
        for pkg in "${@}"; do
            if ! yum list installed "${pkg}" >/dev/null 2>/dev/null; then
                yum install -y "${pkg}"

                purge_list+=( "${pkg}" )
            fi
        done
    fi
}

purge_packages() {
    if (( ${#purge_list[@]} )); then
        if grep -i ubuntu /etc/os-release; then
            apt-get purge --assume-yes --auto-remove "${purge_list[@]}"
        else
            yum remove -y "${purge_list[@]}"
        fi
    fi
}

if_centos() {
    if grep -q -i centos /etc/os-release; then
        eval "${@}"
    fi
}

if_ubuntu() {
    if grep -q -i ubuntu /etc/os-release; then
        eval "${@}"
    fi
}
