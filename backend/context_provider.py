import os
import json
from datetime import datetime, timedelta

from tools.mapbox_client import MapboxClient

# Optional: Google Calendar imports if using Google Calendar API
# from google.oauth2.credentials import Credentials
# from googleapiclient.discovery import build

class ContextProvider:
    """
    Fetches user-specific context data (location, weather, calendar events, personal info),
    and integrates MapboxClient for nearby places and ETA.
    """
    def __init__(self, config_path="config.json"):
        # Load configuration (API keys, file paths, etc.)
        with open(config_path, 'r') as f:
            self.config = json.load(f)

        # Weather API settings
        self.weather_api_key = self.config.get('weather_api_key')
        self.weather_api_url = "https://api.openweathermap.org/data/2.5/weather"

        # Personal info file
        self.personal_info_path = self.config.get('personal_info_path')

        # Calendar settings (stubbed)
        # self.calendar_credentials_path = self.config.get('calendar_credentials_path')

        # Mapbox client
        self.mapbox = MapboxClient(config_path=config_path)

    def get_location(self):
        """
        Returns a dict containing latitude, longitude, and city name.
        """
        return self.config.get('location', {})

    def get_weather(self, latitude, longitude):
        """
        Fetches current weather for given coords.
        """
        import requests
        params = {
            'lat': latitude,
            'lon': longitude,
            'appid': self.weather_api_key,
            'units': self.config.get('default_preferences', {}).get('units', 'imperial')
        }
        resp = requests.get(self.weather_api_url, params=params)
        resp.raise_for_status()
        data = resp.json()
        return {
            'description': data['weather'][0]['description'],
            'temperature': data['main']['temp'],
            'feels_like': data['main']['feels_like']
        }

    def get_calendar_events(self, days=1):
        """
        Fetches upcoming calendar events for the next `days` days.
        This is a stub; integrate your calendar API here.
        """
        now = datetime.utcnow()
        return [
            {
                'title': 'Team Sync',
                'start': (now + timedelta(hours=2)).isoformat() + 'Z',
                'end': (now + timedelta(hours=3)).isoformat() + 'Z'
            }
        ]

    def get_personal_info(self):
        """
        Loads user personal info (name, preferences) from a JSON file.
        """
        if os.path.exists(self.personal_info_path):
            with open(self.personal_info_path, 'r') as f:
                return json.load(f)
        return {}

    def build_context(self, purchase_query: str = None):
        """
        Gathers all context and returns a dict for the LLM.
        If `purchase_query` is provided, includes nearby stores and ETA.
        """
        context = {}

        # Location + Weather
        loc = self.get_location()
        if loc:
            weather = self.get_weather(loc['latitude'], loc['longitude'])
            context['location'] = loc.get('city')
            context['weather'] = weather

        # Calendar
        events = self.get_calendar_events()
        context['upcoming_events'] = events

        # Personal Info
        personal = self.get_personal_info()
        context['user_name'] = personal.get('name')
        context['preferences'] = personal.get('preferences', {})

        # Purchase intent: Mapbox nearby stores & ETA
        if purchase_query:
            # Fetch nearby places
            stores = self.mapbox.search_places(purchase_query, limit=3, radius=self.config.get('default_preferences', {}).get('radius', 5000))
            # Annotate each with ETA
            enriched = []
            for store in stores:
                route = self.mapbox.get_route_info(store['latitude'], store['longitude'])
                eta = None
                if route.get('duration_s'):
                    eta = int(route['duration_s'] / 60)
                store['eta_min'] = eta
                enriched.append(store)
            context['nearby_stores'] = enriched

        return context


if __name__ == "__main__":
    # Example usage
    provider = ContextProvider()
    # Without purchase context
    print(json.dumps(provider.build_context(), indent=2))
    # With purchase context
    print(json.dumps(provider.build_context(purchase_query="Nike Store"), indent=2))
