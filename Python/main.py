import pandas as pd
import os

# Obter o diretório onde está localizado o arquivo main.py
current_dir = os.path.dirname(os.path.abspath(__file__))

# Criar a pasta 'output' caso não exista
output_dir = current_dir + "/output"
os.makedirs(output_dir, exist_ok=True)

# Carregar o dataset original
data = pd.read_csv(current_dir + "/data/traffic_navigation_dataset.csv")

# Criar o conjunto de nodes a partir das colunas 'From' e 'To'
nodes = pd.DataFrame(pd.concat([data['From'], data['To']]).unique(), columns=['Node'])
nodes = nodes.drop_duplicates().reset_index().rename(columns={'index': 'Node_ID'})

# Salvar o arquivo nodes.csv na pasta 'output'
nodes.to_csv(os.path.join(output_dir, "nodes.csv"), index=False)

# Criar um dicionário para mapear cada node ao seu Node_ID
node_id_map = nodes.set_index('Node')['Node_ID'].to_dict()

# Substituir as colunas 'From' e 'To' no dataset original pelo Node_ID correspondente
data['From'] = data['From'].map(node_id_map)
data['To'] = data['To'].map(node_id_map)

# Selecionar as colunas para o arquivo edges.csv
edges = data[['Street', 'From', 'To', 'Distance_km', 'Travel_time_min', 'Congestion_level']]

# Salvar o arquivo edges.csv na pasta 'output'
edges.to_csv(os.path.join(output_dir, "edges.csv"), index=False)

print("Arquivos 'nodes.csv' e 'edges.csv' criados com sucesso na pasta 'output'!")

