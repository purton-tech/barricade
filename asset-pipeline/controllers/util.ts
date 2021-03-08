export function getPassword() : string {
    return localStorage.getItem('password')
}

export function setPassword(password: string) {
    localStorage.setItem('password', password)
}

export function removePassword() {
    localStorage.removeItem('password')
}

export function setPrivateKey(private_key : string) {
    localStorage.setItem('private_key', private_key)
}

export function getPrivateKey() : string {
    return localStorage.getItem('private_key')
}

export function progressFromMap(progressMap : Map<string, number>) : string {

    let list = ""
    progressMap.forEach((value: number, key: string) => {
        let label = key.replace('working-', '')
        const names = label.split('-')
        const capital = names[0].charAt(0).toUpperCase() + names[0].slice(1)
        list += `<p><strong>Field : </strong>${capital} <strong>Search Type</strong> ${names[1]}</p>`
        list += `<progress max="100" value="${value}"></progress>`
    });

    return list;
}

export function percentFromMap(progressMap : Map<string, number>) : number {

    let total = 0
    let length = 0
    progressMap.forEach((value: number, key: string) => {
        total += value
        length += 1
    });
    const percent = total / length
    let percentNearest = Math.ceil(percent / 10) * 10
    return percentNearest
}