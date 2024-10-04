import requests
import threading
from concurrent.futures import ThreadPoolExecutor

# Endpoint para o teste de estresse
url = "http://192.168.137.1:8080/graphs/social/adjacency"

# Função que realiza o GET request
def make_request():
    try:
        response = requests.get(url)
        print(f"Status: {response.status_code}")
    except requests.exceptions.RequestException as e:
        print(f"Error: {e}")

# Função para iniciar as threads
def stress_test(num_requests):
    with ThreadPoolExecutor(max_workers=100) as executor:  # Defina o número de workers conforme a necessidade
        futures = [executor.submit(make_request) for _ in range(num_requests)]
        for future in futures:
            future.result()  # Aguarda que todas as threads terminem

# Número de requisições para o teste de estresse
num_requests = 100000

if __name__ == "__main__":
    stress_test(num_requests)
